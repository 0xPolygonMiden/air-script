use std::collections::HashMap;

use super::MasmVerifierParameters;
use crate::masm::{DOUBLE_WORD_SIZE, generate_with_map_sections};

/// Generates the MASM module of the STARK verifier for handling public inputs.
///
/// There are two main components to this module:
///
/// 1. Fixed length public inputs processing: this takes as input the total number of fixed length
///    public inputs (as base field elements) which is used in order to determine the number of
///    iterations of the loop responsible for loading-storing-hashing the fixed length public
///    inputs,
/// 2. Variable length public inputs processing: this takes as input the map from identifiers to
///    (width, bus_type) of the so-called variable length tables, also called messages widths, and
///    the type of the bus corresponding to each table. This is used in order to generate
///    procedures, one per variable length table, in order to reduce each table, using auxiliary
///    randomness, to an element in the extension field.
pub fn generate_public_inputs(masm_verifier_parameters: &MasmVerifierParameters) -> String {
    let num_iter_load_fixed_len_pub_inputs =
        (masm_verifier_parameters.fixed_len_pub_inputs_total_size()).div_ceil(DOUBLE_WORD_SIZE);

    // procedures to reduce variable length inputs tables, one per table/group
    let mut var_len_pi_reduction_procedures = String::new();
    // code section for calling the above procedures
    let mut reduce_var_len_pi_groups_call = String::new();

    // for each variable length public input group, we create a procedure to reduce the variable
    // length inputs and add a call to the said procedure
    // For each table/group, we associate an label in order to domain separate messages
    // TODO: this will probably be the responsibility of the backend in the near term
    let mut op_batch_section = String::new();
    let mut op_group_label = 0;
    for (identifier, (message_width, bus_type)) in
        masm_verifier_parameters.variable_len_pub_inputs_sizes().iter()
    {
        // create the label for the current group
        op_batch_section += &VAR_LEN_PI_GROUP_OP_LABELS
            .to_string()
            .replace("{GROUP_ID}", &identifier.to_string().to_uppercase())
            .replace("OP_LABEL_VALUE", &op_group_label.to_string().to_uppercase());
        op_group_label += 1;

        // add a call to the procedure for this group
        reduce_var_len_pi_groups_call += &REDUCE_VAR_LEN_PI_GROUP_ID
            .to_string()
            .replace("{group_id}", &identifier.to_string());

        // depending on the type of bus, generate the appropriate procedure for reducing
        // the variable length public inputs
        let procedure = match bus_type {
            air_ir::BusType::Multiset => REDUCE_VAR_LEN_PI_MULTISET_PROCEDURE
                .to_string()
                .replace("{group_id}", &identifier.to_string())
                .replace("{GROUP_ID}", &identifier.to_string().to_uppercase())
                .replace(
                    "{WIDTH_INTERACTION_GROUP_ID_IN_DOUBLE_WORD}",
                    &((*message_width).div_ceil(DOUBLE_WORD_SIZE)).to_string(),
                ),
            air_ir::BusType::Logup => REDUCE_VAR_LEN_PI_LOGUP_PROCEDURE
                .to_string()
                .replace("{group_id}", &identifier.to_string())
                .replace("{GROUP_ID}", &identifier.to_string().to_uppercase())
                .replace(
                    "{WIDTH_INTERACTION_GROUP_ID_IN_DOUBLE_WORD}",
                    &((*message_width).div_ceil(DOUBLE_WORD_SIZE)).to_string(),
                ),
        };
        var_len_pi_reduction_procedures.push_str(&procedure);
    }

    // generate the map for filling the sections
    let mut sections_map = HashMap::new();
    sections_map.insert("DEFINE_VAR_LEN_PI_GROUP_OP_LABELS", op_batch_section);
    sections_map.insert("REDUCE_VAR_LEN_PI_GROUP_ID_CALL", reduce_var_len_pi_groups_call);
    sections_map.insert(
        "REDUCE_VAR_LEN_PI_GROUP_ID_PROCEDURES_DEFINITIONS",
        var_len_pi_reduction_procedures,
    );

    // fill the constants first
    let mut file = PUBLIC_INPUTS_MASM
        .to_string()
        .replace(
            "NUM_FIXED_LEN_PUBLIC_INPUTS_VALUE",
            &masm_verifier_parameters
                .fixed_len_pub_inputs_total_size()
                .next_multiple_of(DOUBLE_WORD_SIZE)
                .to_string(),
        )
        .replace("NUM_VAR_LEN_PI_GROUPS_VALUE", &op_group_label.to_string())
        .replace(
            "NUM_ITER_LOAD_FIXED_LEN_PUB_INPUTS",
            &num_iter_load_fixed_len_pub_inputs.to_string(),
        );

    // then we fill the sections
    generate_with_map_sections(&mut file, sections_map);

    file
}

// TEMPLATES
// ================================================================================================

const PUBLIC_INPUTS_MASM: &str = r#"
use.std::crypto::stark::constants
use.std::crypto::stark::random_coin
use.std::crypto::stark::public_inputs

use.std::crypto::hashes::rpo

# CONSTANTS
# =================================================================================================

# Number of fixed length public inputs with padding (in field elements)
const.NUM_FIXED_LEN_PUBLIC_INPUTS=NUM_FIXED_LEN_PUBLIC_INPUTS_VALUE

# Number of variable length public input groups
const.NUM_VAR_LEN_PI_GROUPS=NUM_VAR_LEN_PI_GROUPS_VALUE

# Op label for variable length public input groups
# BEGIN_SECTION:DEFINE_VAR_LEN_PI_GROUP_OP_LABELS
# END_SECTION:DEFINE_VAR_LEN_PI_GROUP_OP_LABELS

# CONSTANTS GETTERS
# =================================================================================================

export.get_num_fixed_len_public_inputs
    push.NUM_FIXED_LEN_PUBLIC_INPUTS
end

# MAIN PROCEDURE
# =================================================================================================

#! Processes the public inputs.
#! 
#! This involves:
#!
#! 1. Loading from the advice stack the fixed-length public inputs and storing them in memory
#!    starting from the address pointed to by `public_inputs_address_ptr`.
#! 2. Loading from the advice stack the variable-length public inputs, storing them temporarily
#!    in memory, and then reducing them to an element in the challenge field using the auxiliary
#!    randomness. This reduced value is then used to impose a boundary condition on the relevant
#!    auxiliary column. 
#!
#! Note that the fixed length public inputs are stored as extension field elements while
#! the variable length ones are stored as base field elements.
#!
#! Note also that, while loading the above, we compute the hash of the public inputs. The hashing
#! starts with capacity registers of the hash function set to `C` that is the result of hashing
#! the proof context.
#!
#! The output D, that is the digest of the above hashing, is then used in order to reseed
#! the random coin.
#!
#! It is worth noting that:
#!
#! 1. Only the fixed-length public inputs are stored for the lifetime of the verification procedure.
#!    The variable-length public inputs are stored temporarily, as this simplifies the task of
#!    reducing them using the auxiliary randomness. On the other hand, the resulting values from
#!    the aforementioned reductions are stored right after the fixed-length public inputs. These
#!    are stored in a word-aligned manner and padded with zeros if needed.
#! 2. The public inputs address is computed in such a way so as we end up with the following
#!    memory layout:
#!
#!    [..., a_0...a_{m-1}, b_0...b_{n-1}, alpha0, alpha1, beta0, beta1, OOD-evaluations-start, ...]
#!
#!    where:
#!
#!    1. [a_0...a_{m-1}] are the fixed-length public inputs stored as extension field elements. This
#!       section is double-word-aligned.
#!    2. [b_0...b_{n-1}] are the results of reducing the variable length public inputs using
#!       auxiliary randomness. This section is word-aligned.
#!    3. [alpha0, alpha1, beta0, beta1] is the auxiliary randomness.
#!    4. `OOD-evaluations-start` is the first field element of the section containing the OOD
#!       evaluations.
#! 3. Note that for each bus message in a group in the variable length public inputs, each
#!    message is expected to be padded to the next multiple of 8 and provided in reverse order.
#!    This has the benefit of making the reduction using the auxiliary randomness more efficient
#!    using `horner_eval_base`.
#!
#!
#! Input: [C, ...]
#! Output: [...]
export.process_public_inputs
    # 1) Compute the address where the public inputs will be stored and store it.
    #    This also computes the address where the reduced variable-length public inputs will be stored.
    exec.get_num_fixed_len_public_inputs push.NUM_VAR_LEN_PI_GROUPS
    exec.public_inputs::compute_and_store_public_inputs_address
    # => [C, ...]

    # 2) Load the public inputs.
    #    This will also hash them so that we can absorb them in the transcript.
    exec.load_public_inputs
    # => [D, ...]

    # 3) Absorb into the transcript
    exec.random_coin::reseed
    # => [...]

    # 4) Reduce the variable-length public inputs using randomness.
    exec.reduce_variable_length_public_inputs
end

# HELPER PROCEDURES
# =================================================================================================

#! Loads from the advice stack the public inputs and stores them in memory starting from address
#! pointed to by `public_inputs_address_ptr`.
#!
#! Note that the public inputs are stored as extension field elements.
#!
#! In parallel, it computes the hash of the public inputs being loaded. The hashing starts with
#! capacity registers of the hash function set to `C` resulting from hashing the proof context.
#! The output D is the digest of the hashing of the public inputs.
#!
#! Inputs:  [C, ...]
#! Outputs: [D, ...]
proc.load_public_inputs
    # 1) Load and hash the fixed length public inputs
    
    exec.constants::public_inputs_address_ptr mem_load
    movdn.4
    padw padw
    repeat.NUM_ITER_LOAD_FIXED_LEN_PUB_INPUTS
        exec.public_inputs::load_base_store_extension_double_word
        hperm
    end

    # 2) Load and hash the variable length public inputs

    ## a) Compute the number of base field elements in total in the variable length public inputs
    exec.constants::num_public_inputs_ptr mem_load
    exec.get_num_fixed_len_public_inputs
    sub
    # => [num_var_len_pi, R2, R1, C, ptr, ...]

    ## b) Compute the number of hash iteration needed to hash the variable length public inputs.
    ##    We also check the double-word alignment.
    u32divmod.8
    # => [rem, num_iter, R2, R1, C, ptr, ...]
    push.0 assert_eq
    # => [num_iter, R2, R1, C, ptr, ...]
    
    ## c) Prepare the stack for hashing
    movdn.13
    # => [R2, R1, C, ptr, num_iter, ...]
    dup.13 sub.1 swap.14
    push.0 neq
    # => [(num_iter == 0), R2, R1, C, ptr, num_iter - 1, ...]

    ## d) Hash the variable length public inputs
    while.true
        adv_pipe
        hperm
        # => [R2, R1, C, ptr, num_iter, ...]
        dup.13 sub.1 swap.14
        push.0 neq
    end
    # => [R2, R1, C, ptr, num_iter, ...]

    # 3) Return the final digest
    exec.rpo::squeeze_digest
    # => [D, ptr, num_iter, ...] where D = R1 the digest
    movup.4 drop
    movup.4 drop
    # => [D, ...]
end

#! Reduces the variable-length public inputs using the auxiliary randomness.
#!
#! The procedure non-deterministically loads the auxiliary randomness from the advice tape and
#! stores it at `aux_rand_nd_ptr` so that it can be later checked for correctness. After this,
#! the procedure uses the auxiliary randomness in order to reduce the variable-length public
#! inputs to a single element in the challenge field. The resulting values are then stored
#! contiguously after the fixed-length public inputs.
#!
#! Input: 
#!      - Operand stack: [...]
#!      - Advice stack: [beta0, beta1, alpha0, alpha1, var_len_pi_1_len, ..., var_len_pi_k_len, ...]
#! Output: [D, ...]
proc.reduce_variable_length_public_inputs
    # 1) Load the auxiliary randomness i.e., alpha and beta
    #    We store them as [beta0, beta1, alpha0, alpha1] since `horner_eval_ext` requires memory
    #    word-alignment.
    adv_push.4
    exec.constants::aux_rand_nd_ptr mem_storew
    # => [alpha1, alpha0, beta1, beta0, ...]
    dropw
    # => [...]

    # 2) Get the pointer to the variable-length public inputs.
    #    This is also the pointer to the first address at which we will store the results of
    #    the reductions.
    exec.constants::variable_length_public_inputs_address_ptr mem_load
    dup
    # => [next_var_len_pub_inputs_ptr, var_len_pub_inputs_res_ptr, ...] where
    # `next_var_len_pub_inputs_ptr` points to the next chunk of variable public inputs to be reduced,
    # and `var_len_pub_inputs_res_ptr` points to the next available memory location where the result
    # of the reduction can be stored.
    # Note that, as mentioned in the top of this module, the variable-length public inputs are only
    # stored temporarily and they will be over-written by, among other data, the result of reducing
    # the variable public inputs.

    # BEGIN_SECTION:REDUCE_VAR_LEN_PI_GROUP_ID_CALL
    # END_SECTION:REDUCE_VAR_LEN_PI_GROUP_ID_CALL

    # 3) Clean up the stack.
    drop drop
    # => [...]
end

# BEGIN_SECTION:REDUCE_VAR_LEN_PI_GROUP_ID_PROCEDURES_DEFINITIONS
# END_SECTION:REDUCE_VAR_LEN_PI_GROUP_ID_PROCEDURES_DEFINITIONS

"#;

const VAR_LEN_PI_GROUP_OP_LABELS: &str = r#" 
const.VAR_LEN_PI_GROUP_{GROUP_ID}_OP_LABEL=OP_LABEL_VALUE
"#;

const REDUCE_VAR_LEN_PI_GROUP_ID: &str = r#"adv_push.1 exec.reduce_var_len_pi_group_{group_id}
# => [next_var_len_pub_inputs_ptr, var_len_pub_inputs_res_ptr, ...]
"#;

const REDUCE_VAR_LEN_PI_MULTISET_PROCEDURE: &str = r#" 

#! Reduces the variable length public inputs for this group using auxiliary randomness.
#!
#! Inputs:  [num_interaction, interaction_ptr]
#! Outputs: [next_ptr]
#!
#! where `interaction_ptr` is a pointer to the messages in this group.
proc.reduce_var_len_pi_group_{group_id}
    # Assert that the number of interactions is at most 1023
    dup u32lt.1024 assert

    # Store number of interactions
    push.0.0 dup.2
    exec.constants::tmp1 mem_storew
    # => [num_interaction, 0, 0, num_interaction, interaction_ptr, ...]

    # Load alpha
    exec.constants::aux_rand_nd_ptr mem_loadw
    # => [alpha1, alpha0, beta1, beta0, interaction_ptr, ...]

    # We will keep [beta0, beta1, alpha0 + op_label, alpha1] on the stack so that we can compute
    # the final result, where op_label is a unique label to domain separate the interaction with
    # the chiplets` bus.
    # The final result is then computed as:
    #
    #   alpha + op_label * beta^0 + beta * (r_0 * beta^0 + r_1 * beta^1 + r_2 * beta^2 + r_3 * beta^3)
    swap
    push.VAR_LEN_PI_GROUP_{GROUP_ID}_OP_LABEL
    add
    swap
    # => [alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, ...]

    # Push the `horner_eval_ext` accumulator
    push.0.0
    # => [acc1, acc0, alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, ...]

    # Push the pointer to the evaluation point beta
    exec.constants::aux_rand_nd_ptr
    # => [beta_ptr, acc1, acc0, alpha1, alpha0 + op_label, beta1, beta0,  interaction_ptr, ...]

    # Get the pointer to interactions
    movup.7
    # => [interaction_ptr, beta_ptr, acc1, acc0, alpha1, alpha0 + op_label, beta1, beta0,  ...]

    # Set up the stack for `mem_stream` + `horner_eval_ext`
    swapw
    padw padw
    # => [Y, Y, alpha1, alpha0 + op_label, beta1, beta0,  interaction_ptr, beta_ptr, acc1, acc0, ...]
    # where `Y` is a garbage word.

    exec.constants::tmp1 mem_loadw dup
    push.0
    neq

    while.true
        repeat.{WIDTH_INTERACTION_GROUP_ID_IN_DOUBLE_WORD}
            mem_stream
            horner_eval_base
        end
        # => [Y, Y, alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, acc1, acc0, ...]

        swapdw
        # => [alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, acc1, acc0, Y, Y, ...]

        movup.7 movup.7
        # => [acc1, acc0, alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, Y, Y, ...]
        
        dup.5 dup.5
        # => [beta1, beta0, acc1, acc0, alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, Y, Y, ...]
        ext2mul
        # => [tmp1', tmp0', alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, Y, Y, ...]

        dup.3 dup.3
        ext2add
        # => [term1', term0', alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, Y, Y, ...]
  
        movdn.15
        movdn.15
        # => [alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, Y, Y, term1', term0', ...]

        push.0 movdn.6
        push.0 movdn.6
        # => [alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, 0, 0, Y, Y, term1', term0', ...]
 
        swapdw
        # => [Y, Y, alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, 0, 0, term1', term0', ...]

        exec.constants::tmp1 mem_loadw sub.1
        exec.constants::tmp1 mem_storew
 
        dup
        push.0
        neq
    end
    # => [Y, Y, alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, 0, 0, term1', term0', ...]

    dropw dropw dropw
    # => [interaction_ptr, beta_ptr, 0, 0, term1', term0', ...]
    dup exec.constants::tmp2 mem_store
    exec.constants::tmp1 mem_loadw drop drop drop

    push.1.0
    movup.2
    dup
    push.0
    neq
    # => [loop, n, acc1, acc0, term1_1, term1_0, ..., termn_1, termn_0, ...]

    while.true
        sub.1 movdn.4
        # => [acc1, acc0, term1_1, term1_0, n - 1, ..., termn_1, termn_0, ...]
        ext2mul
        # => [acc1', acc0', n - 1, ..., termn_1, termn_0, ...]
        movup.2
        dup
        push.0
        neq
        # => [loop, n - 1, acc1', acc0', term1_1, term1_0, ..., termn_1, termn_0, ...]
    end

    drop
    exec.constants::tmp2 mem_load movdn.2
    # since we are initializing the bus with "requests", we should invert the reduced result
    ext2inv
    # => [prod_acc1, prod_acc0, interaction_ptr, ...]

    # Store the result
    push.0.0
    # => [0, 0, prod_acc1, prod_acc0, interaction_ptr, var_len_pub_inputs_res_ptr, ...]
    dup.5 add.4 swap.6
    mem_storew
    dropw
    # => [interaction_ptr, var_len_pub_inputs_res_ptr, ...]
end
"#;

const REDUCE_VAR_LEN_PI_LOGUP_PROCEDURE: &str = r#" 
#! Reduces the variable length public inputs for this group using auxiliary randomness.
#!
#! Inputs:  [num_interaction, interaction_ptr]
#! Outputs: [next_ptr]
#!
#! where `interaction_ptr` is a pointer to the messages in this group.
proc.reduce_var_len_pi_group_{group_id}
    # Assert that the number of interactions is at most 1023
    dup u32lt.1024 assert

    # Store number of interactions
    push.0.0 dup.2
    exec.constants::tmp1 mem_storew
    # => [num_interaction, 0, 0, num_interaction, interaction_ptr, ...]

    # Load alpha
    exec.constants::aux_rand_nd_ptr mem_loadw
    # => [alpha1, alpha0, beta1, beta0, interaction_ptr, ...]

    # We will keep [beta0, beta1, alpha0 + op_label, alpha1] on the stack so that we can compute
    # the final result, where op_label is a unique label to domain separate the interaction with
    # the chiplets` bus.
    # The final result is then computed as:
    #
    #   alpha + op_label * beta^0 + beta * (r_0 * beta^0 + r_1 * beta^1 + r_2 * beta^2 + r_3 * beta^3)
    swap
    push.VAR_LEN_PI_GROUP_{GROUP_ID}_OP_LABEL
    add
    swap
    # => [alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, ...]

    # Push the `horner_eval_ext` accumulator
    push.0.0
    # => [acc1, acc0, alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, ...]

    # Push the pointer to the evaluation point beta
    exec.constants::aux_rand_nd_ptr
    # => [beta_ptr, acc1, acc0, alpha1, alpha0 + op_label, beta1, beta0,  interaction_ptr, ...]

    # Get the pointer to interactions
    movup.7
    # => [interaction_ptr, beta_ptr, acc1, acc0, alpha1, alpha0 + op_label, beta1, beta0,  ...]

    # Set up the stack for `mem_stream` + `horner_eval_ext`
    swapw
    padw padw
    # => [Y, Y, alpha1, alpha0 + op_label, beta1, beta0,  interaction_ptr, beta_ptr, acc1, acc0, ...]
    # where `Y` is a garbage word.

    exec.constants::tmp1 mem_loadw dup
    push.0
    neq

    while.true
        repeat.{WIDTH_INTERACTION_GROUP_ID_IN_DOUBLE_WORD}
            mem_stream
            horner_eval_base
        end
        # => [Y, Y, alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, acc1, acc0, ...]

        swapdw
        # => [alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, acc1, acc0, Y, Y, ...]

        movup.7 movup.7
        # => [acc1, acc0, alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, Y, Y, ...]
        
        dup.5 dup.5
        # => [beta1, beta0, acc1, acc0, alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, Y, Y, ...]
        ext2mul
        # => [tmp1', tmp0', alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, Y, Y, ...]

        dup.3 dup.3
        ext2add
        ext2inv
        # => [term1', term0', alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, Y, Y, ...]
  
        movdn.15
        movdn.15
        # => [alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, Y, Y, term1', term0', ...]

        push.0 movdn.6
        push.0 movdn.6
        # => [alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, 0, 0, Y, Y, term1', term0', ...]
 
        swapdw
        # => [Y, Y, alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, 0, 0, term1', term0', ...]

        exec.constants::tmp1 mem_loadw sub.1
        exec.constants::tmp1 mem_storew
 
        dup
        push.0
        neq
    end
    # => [Y, Y, alpha1, alpha0 + op_label, beta1, beta0, interaction_ptr, beta_ptr, 0, 0, term1', term0', ...]

    dropw dropw dropw
    # => [interaction_ptr, beta_ptr, 0, 0, term1', term0', ...]
    dup exec.constants::tmp2 mem_store
    exec.constants::tmp1 mem_loadw drop drop drop

    push.0.0
    movup.2
    dup
    push.0
    neq
    # => [loop, n, acc1, acc0, term1_1, term1_0, ..., termn_1, termn_0, ...]

    while.true
        sub.1 movdn.4
        # => [acc1, acc0, term1_1, term1_0, n - 1, ..., termn_1, termn_0, ...]
        ext2add
        # => [acc1', acc0', n - 1, ..., termn_1, termn_0, ...]
        movup.2
        dup
        push.0
        neq
        # => [loop, n - 1, acc1', acc0', term1_1, term1_0, ..., termn_1, termn_0, ...]
    end

    drop
    exec.constants::tmp2 mem_load movdn.2
    # since we are initializing the bus with "requests", we should negate the reduced result
    ext2neg
    # => [sum_acc1, sum_acc0, interaction_ptr, ...]

    # Store the result
    push.0.0
    # => [0, 0, sum_acc1, sum_acc0, interaction_ptr, var_len_pub_inputs_res_ptr, ...]
    dup.5 add.4 swap.6
    mem_storew
    dropw
    # => [interaction_ptr, var_len_pub_inputs_res_ptr, ...]
end
"#;
