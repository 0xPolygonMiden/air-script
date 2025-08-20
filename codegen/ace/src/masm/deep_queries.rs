use std::collections::HashMap;

use crate::masm::{
    DOUBLE_WORD_SIZE, FIELD_EXTENSION_DEGREE, MasmVerifierParameters, generate_with_map_sections,
};

/// Generates the MASM module of the STARK verifier for computing the queries to the DEEP
/// composition polynomial.
///
/// The main logic for generating the module needs to determine the need for adding a section
/// to process the part of the query related to the auxiliary trace in addition to the sizes
/// of each of the three sections, namely the main, auxilary and constraints composition polynomials
/// trace sections. Once this is determined, we can assign the values to the loops
/// processing each of the sections involved in the query computation.
pub fn generate_deep_queries_module(masm_verifier_parameters: &MasmVerifierParameters) -> String {
    // compute the constants related to the processing of the main segment and constraint
    // segment traces
    let num_iterations_main_trace_part =
        (masm_verifier_parameters.main_trace_width() as usize).div_ceil(DOUBLE_WORD_SIZE);
    let num_iterations_cc_trace_part =
        (masm_verifier_parameters.constraints_composition_trace_width() * FIELD_EXTENSION_DEGREE)
            .div_ceil(DOUBLE_WORD_SIZE);

    // fill the template with the computed constants
    let mut file = DEEP_QUERIES_MASM
        .to_string()
        .replace("NUM_ITERATIONS_MAIN_TRACE_PART", &num_iterations_main_trace_part.to_string())
        .replace(
            "NUM_ITERATIONS_CONSTRAINTS_COMPOSITION_TRACE_PART",
            &num_iterations_cc_trace_part.to_string(),
        );

    // handle the case when auxiliary segment exists
    if let Some(aux_trace_width) = masm_verifier_parameters.aux_trace_width {
        // first, build the section for processing the auxiliary segment, which requires computing
        // the number of iterations in its main loop
        let num_iterations_aux_trace_part =
            (aux_trace_width as usize * FIELD_EXTENSION_DEGREE).div_ceil(DOUBLE_WORD_SIZE);
        let aux_trace_part_processing = AUX_TRACE_PART_PROCESSING
            .to_string()
            .replace("NUM_ITERATIONS_AUX_TRACE_PART", &num_iterations_aux_trace_part.to_string());

        // add a call to the generated procedure and include the generated procedure in the main
        // file
        let mut sections_map = HashMap::new();
        sections_map.insert("PROCESS_AUX_TRACE_PART_CALL", PROCESS_AUX_TRACE_PART_CALL.to_string());
        sections_map
            .insert("IMPL_PROCESS_AUX_TRACE_PART_CALL", aux_trace_part_processing.to_string());
        generate_with_map_sections(&mut file, sections_map);
    }

    file
}

// TEMPLATES
// ================================================================================================

const DEEP_QUERIES_MASM: &str = r#"
use.std::crypto::stark::constants
use.std::crypto::stark::deep_queries

# ERRORS
# =================================================================================================

const.LEAF_VALUE_MISMATCH="hash of leaf pre-image does not match leaf value from the Merkle tree"

# MAIN PROCEDURE
# =================================================================================================

#! Computes the DEEP composition polynomial FRI queries.
#!
#! This procedures iterates over all FRI query indices stored in memory at `query_ptr` in
#! a word-aligned and overwrites each word with `[eval0, eval1, index, poe]` where:
#!
#! 1. `index` is the FRI query index,
#! 2. `poe := g^index`, with `g` being the evaluation domain generator,
#! 3. `eval := (eval0, eval1)` is the computed DEEP composition polynomial query.
#!
#! Inputs:  [Y, query_ptr, query_end_ptr, W, query_ptr]
#! Outputs: []
#!
#! where:
#!
#! 1. `Y` is a garbage word,
#! 2. `query_ptr` is a pointer to the memory region from where the query indices will be fetched
#!    and to where the computed FRI queries will be stored in a word-aligned manner,
#! 3. `query_end_ptr` is a memory pointer used to indicate the end of the memory region used in
#!    storing the computed FRI queries,
#! 4. `W` is the word `[q_z_0, q_z_1, q_gz_0, q_gz_1]` where `q_z = (q_z_0, q_z_1)` and
#!    `q_gz = (q_gz_0, q_gz_1)` represent the constant terms across all FRI queries computations.
export.compute_deep_composition_polynomial_queries

    # Iterate over all FRI query indices and compute their correspond DEEP query
    # The following assumes the existence of at least one query to compute which is a necessary
    # requirement to get any soundness from the protocol. This assumption is validate at
    # the start of the verification procedure.
    push.1
    while.true
        # Load the (main, aux, constraint)-traces rows associated with the current query and get
        # the index of the query.
        exec.load_query_row
        #=> [Y, X, index, query_ptr, query_end_ptr, W, query_ptr]

        # Compute the current query and store the result
        # We also re-arrange the stack for the next iteration of the loop
        exec.deep_queries::compute_deep_query
        # => [has_more_queries, Y, query_ptr+1, query_end_ptr]
    end

    # Clean up the stack and return
    dropw dropw drop drop drop
end

# HELPER PROCEDURES
# =================================================================================================

#! Loads the next query rows in the main, auxiliary and constraint composition polynomials traces
#! and computes the values of the DEEP code word at the index corresponding to the query.
#!
#! It takes a pointer to the current random query index and returns that index, together with
#! the value
#!
#! Q^x(alpha) = (q_x_at_alpha_0, q_x_at_alpha_1) = \sum_{i=0}^{n+m+l} T_i * alpha^i
#!
#! where:
#!
#! 1. n, m and l are the widths of the main segment, auxiliary segment and constraint composition
#!    traces, respectively.
#! 2. T_i are the values of columns in the main segment, auxiliary segment and constraint
#!    composition traces, for the query.
#! 3. alpha is the randomness used in order to build the DEEP polynomial.
#!
#! Inputs:  [Y, query_ptr]
#! Outputs: [Y, q_x_at_alpha_1, q_x_at_alpha_0, q_x_at_alpha_1, q_x_at_alpha_0, index, query_ptr]
#!
#! where:
#! - Y is a "garbage" word.
proc.load_query_row
    # Process the main segment of the execution trace portion of the query
    exec.process_main_segment_execution_trace
    #=> [Y, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]

    # BEGIN_SECTION:PROCESS_AUX_TRACE_PART_CALL
    # END_SECTION:PROCESS_AUX_TRACE_PART_CALL

    # Process the constraints composition polys trace portion of the query
    exec.process_constraints_composition_poly_trace
    #=> [Y, q_x_at_alpha_1, q_x_at_alpha_0, q_x_at_alpha_1, q_x_at_alpha_0, index, query_ptr]
end

# MAIN TRACE SEGMENT PROCESSING
# =================================================================================================

#! Handles the logic for processing the main segment of the execution trace.
#! 
#! Inputs: [Y, query_ptr]
#! Output: [Y, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]
proc.process_main_segment_execution_trace
    # Load the query index
    dup.4 mem_loadw
    #=> [index, depth, y, y, query_ptr] where y are "garbage" values here and throughout

    # Get commitment to main segment of the execution trace
    movdn.3 movdn.2
    push.0.0
    exec.constants::main_trace_com_ptr mem_loadw
    #=>[MAIN_TRACE_TREE_ROOT, depth, index, query_ptr]

    # Use the commitment to get the leaf and save it
    dup.5 dup.5
    mtree_get
    exec.constants::tmp3 mem_storew
    adv.push_mapval
    #=>[LEAF_VALUE, MAIN_TRACE_TREE_ROOT, depth, index, query_ptr]

    exec.constants::tmp2 mem_loadw
    swapw
    #=>[LEAF_VALUE, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]

    # Load the values of the main segment of the execution trace at the current query. We also
    # compute their hashing and the value of their random linear combination using powers of a
    # single random value alpha.
    padw swapw padw
    #=> [Y, Y, 0, 0, 0, 0, ptr, y, y, y]
    exec.load_main_segment_execution_trace
    #=> [Y, L, C, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]
 
    # Load the leaf value we got using `mtree_get` and compare it against the hash we just computed
    exec.constants::tmp3 mem_loadw
    assert_eqw.err=LEAF_VALUE_MISMATCH
end

#! Loads the portion of the query associated to the main segment of the execution trace.
#!
#! Inputs:  [Y, Y, 0, 0, 0, 0, ptr]
#! Outputs: [Y, D, C, ptr]
proc.load_main_segment_execution_trace
    repeat.NUM_ITERATIONS_MAIN_TRACE_PART
        adv_pipe
        horner_eval_base
        hperm
    end
end

# BEGIN_SECTION:IMPL_PROCESS_AUX_TRACE_PART
# END_SECTION:IMPL_PROCESS_AUX_TRACE_PART

# CONSTRAINTS COMPOSITION POLYNOMIALS TRACE PROCESSING
# =================================================================================================

#! Handles the logic for processing the constraints composition polynomials trace.
#! 
#! Inputs: [Y, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]
#! Output: [Y, q_x_at_alpha_1, q_x_at_alpha_0, q_x_at_alpha_1, q_x_at_alpha_0, index, query_ptr]
proc.process_constraints_composition_poly_trace
    # Load the commitment to the constraint trace
    exec.constants::composition_poly_com_ptr mem_loadw
    #=> [R, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]

    # Get the leaf against the commitment
    dup.9 movup.9
    mtree_get
    exec.constants::tmp3 mem_storew
    adv.push_mapval
    #=>[L, R, ptr_x, ptr_alpha_inv, acc1, acc0, index, query_ptr]

    # Load the 8 columns as quadratic extension field elements in batches of 4. 
    padw
    swapw.2
    exec.load_constraints_composition_polys_trace
    #=> [Y, L, Y, ptr_x, ptr_alpha_inv, acc1, acc0, index, query_ptr]

    # Load the leaf value we got using `mtree_get` and compare it against the hash we just computed
    exec.constants::tmp3 mem_loadw
    assert_eqw.err=LEAF_VALUE_MISMATCH
    #=> [Y, ptr_x, ptr_alpha_inv, acc1, acc0, index, query_ptr]

    # Re-order the stack
    swapw
    drop drop
    dup.1 dup.1
    swapw
end

#! Loads the portion of the query associated to the constraints composition polynomials trace.
#!
#! Inputs:  [Y, Y, 0, 0, 0, 0, ptr]
#! Outputs: [Y, D, C, ptr]
proc.load_constraints_composition_polys_trace
    repeat.NUM_ITERATIONS_CONSTRAINTS_COMPOSITION_TRACE_PART
        adv_pipe
        horner_eval_ext        
        hperm
    end
end
"#;

const PROCESS_AUX_TRACE_PART_CALL: &str = r#" 
# Process the auxiliary segment of the execution trace portion of the query
exec.process_aux_segment_execution_trace
#=> [Y, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]
"#;

const AUX_TRACE_PART_PROCESSING: &str = r#"
# AUX TRACE SEGMENT PROCESSING
# =================================================================================================

#! Handles the logic for processing the auxiliary segment of the execution trace, if such a trace
#! exists.
#! 
#! Inputs: [Y, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]
#! Output: [Y, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]
proc.process_aux_segment_execution_trace
    # Load aux trace commitment and get leaf
    exec.constants::aux_trace_com_ptr mem_loadw

    # Get the leaf against the auxiliary trace commitment for the current query
    dup.9 dup.9
    mtree_get
    exec.constants::tmp3 mem_storew
    adv.push_mapval
    #=> [L, R, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]

    # Load the values of the auxiliary segment of the execution trace at the current query
    
    # Set up the stack
    exec.constants::zero_word_ptr mem_loadw
    swapw padw
    #=> [Y, Y, C, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]

    # Load the first 4 columns as a batch of 4 quadratic extension field elements.
    exec.load_aux_segment_execution_trace
    #=> [Y, D, C, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]

    # Load the leaf value we got using `mtree_get` and compare it against the hash we just computed
    exec.constants::tmp3 mem_loadw
    assert_eqw.err=LEAF_VALUE_MISMATCH
    #=> [Y, ptr_x, ptr_alpha_inv, acc1, acc0, depth, index, query_ptr]
end

#! Loads the portion of the query associated to the auxiliary segment of the execution trace.
#!
#! Inputs:  [Y, Y, 0, 0, 0, 0, ptr]
#! Outputs: [Y, D, C, ptr]
proc.load_aux_segment_execution_trace
    repeat.NUM_ITERATIONS_AUX_TRACE_PART
        adv_pipe
        horner_eval_ext      
        hperm
    end
end
"#;
