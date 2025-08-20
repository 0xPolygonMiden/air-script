use std::ops::Div;

use crate::{
    EncodedAceCircuit,
    masm::{DOUBLE_WORD_SIZE, MasmVerifierParameters, add_section},
};

/// Generates the MASM module of the STARK verifier for checking constraints evaluation.
///
/// The main logic for generating the module needs to determine the parameters to the `eval_circuit`
/// op, the arithmetic circuit description, and the number of iterations required for hashing it.
pub fn generate_constraints_eval_module(
    masm_verifier_parameters: &MasmVerifierParameters,
    encoded_circuit: &EncodedAceCircuit,
) -> String {
    // generate the constants
    let num_inputs_circuit = encoded_circuit.num_vars();
    let num_eval_gates_circuit = encoded_circuit.num_eval_rows();
    let circuit_description = encoded_circuit.instructions();
    let num_iterations_hash_ace_circuit = circuit_description.len().div(DOUBLE_WORD_SIZE);
    assert_eq!(circuit_description.len() % DOUBLE_WORD_SIZE, 0);

    // fill the template with the generated constants
    let mut file = CONSTRAINTS_EVAL_MASM
        .to_string()
        .replace("NUM_CONSTRAINTS_VALUE", &masm_verifier_parameters.num_constraints().to_string())
        .replace(
            "MAX_CYCLE_LEN_LOG_VALUE",
            &masm_verifier_parameters.max_cycle_len_log().to_string(),
        )
        .replace("NUM_INPUTS_CIRCUIT_VALUE", &num_inputs_circuit.to_string())
        .replace("NUM_ITERATIONS_HASH_ACE_CIRCUIT", &num_iterations_hash_ace_circuit.to_string())
        .replace("NUM_EVAL_GATES_CIRCUIT_VALUE", &num_eval_gates_circuit.to_string());

    // generate the section containing the circuit description for loading into the advice map
    let advice_map_circuit_description_section = format_circuit_description(encoded_circuit);

    // add the generated section to the template file
    add_section(
        &mut file,
        "ADVICE_MAP_CIRCUIT_DESCRIPTION",
        &advice_map_circuit_description_section,
    );

    file
}

// HELPERS
// ================================================================================================

/// Formats the circuit description using the syntax for loading static data through the advice map.
fn format_circuit_description(circuit: &EncodedAceCircuit) -> String {
    let circuit_digest = circuit.circuit_hash();
    let mut result: String =
        format!("adv_map.CIRCUIT_COMMITMENT(0x{circuit_digest})=[\n").to_string();
    result += &circuit
        .instructions()
        .iter()
        .map(|elem| format!("{}", elem.as_int()))
        .collect::<Vec<_>>()
        .join(",\n");
    result += "\n]\n";

    result
}

// TEMPLATES
// ================================================================================================

const CONSTRAINTS_EVAL_MASM: &str = r#"
use.std::crypto::hashes::rpo
use.std::crypto::stark::constants
use.std::crypto::stark::utils

# CONSTANTS
# =================================================================================================

# Number of constraints, both boundary and transitional
const.NUM_CONSTRAINTS=NUM_CONSTRAINTS_VALUE

# Number of inputs to the constraint evaluation circuit
const.NUM_INPUTS_CIRCUIT=NUM_INPUTS_CIRCUIT_VALUE

# Number of evaluation gates in the constraint evaluation circuit
const.NUM_EVAL_GATES_CIRCUIT=NUM_EVAL_GATES_CIRCUIT_VALUE

# Max cycle length for periodic columns
const.MAX_CYCLE_LEN_LOG=MAX_CYCLE_LEN_LOG_VALUE

# --- constant getters --------------------------------------------------------

proc.get_num_constraints
    push.NUM_CONSTRAINTS
end

# ERRORS
# =================================================================================================

const.ERR_FAILED_TO_LOAD_CIRCUIT_DESCRIPTION="failed to load the circuit description for the constraints evaluation check"


# CONSTRAINT EVALUATION CHECKER
# =================================================================================================

#! Executes the constraints evaluation check by evaluating an arithmetic circuit using the ACE
#! chiplet.
#!
#! The circuit description is hardcoded into the verifier using its commitment, which is computed as
#! the sequential hash of its description using RPO hasher. The circuit description, containing both
#! constants and evaluation gates description, is stored at the contiguous memory region starting
#! at `ACE_CIRCUIT_PTR`. The variable part of the circuit input is stored at the contiguous memory
#! region starting at `pi_ptr`. The (variable) inputs to the circuit are laid out such that the
#! aforementioned memory regions are together contiguous with the (variable) inputs section.
#!
#! Inputs:  []
#! Outputs: []
export.execute_constraint_evaluation_check
    # Compute and store at the appropriate memory location the auxiliary inputs needed by
    # the arithmetic circuit.
    push.MAX_CYCLE_LEN_LOG
    exec.utils::set_up_auxiliary_inputs_ace
    # => []

    # Load the circuit description from the advice tape and check that it matches the hardcoded digest
    exec.load_ace_circuit_description
    # => []

    # Set up the inputs to the "eval_circuit" op. Namely:
    # 1. a pointer to the inputs of the circuit in memory,
    # 2. the number of inputs to the circuit,
    # 3. the number of evaluation gates in the circuit.
    push.NUM_EVAL_GATES_CIRCUIT
    push.NUM_INPUTS_CIRCUIT
    exec.constants::public_inputs_address_ptr mem_load
    # => [pi_ptr, n_read, n_eval]

    # Perform the constraint evaluation check by checking that the circuit evaluates to zero, which
    # boils down to the "eval_circuit" returning.
    eval_circuit
    # => [pi_ptr, n_read, n_eval]

    # Clean up the stack.
    drop drop drop
    # => []
end

#! Loads the description of the ACE circuit for the constraints evaluation check.
#!
#! Inputs:  []
#! Outputs: []
proc.load_ace_circuit_description
    push.CIRCUIT_COMMITMENT
    adv.push_mapval
    exec.constants::get_arithmetic_circuit_ptr
    padw padw padw
    repeat.NUM_ITERATIONS_HASH_ACE_CIRCUIT
        adv_pipe
        hperm
    end
    exec.rpo::squeeze_digest
    movup.4 drop
    assert_eqw.err=ERR_FAILED_TO_LOAD_CIRCUIT_DESCRIPTION
    # => []
end


# CONSTRAINT EVALUATION CIRCUIT DESCRIPTION
# =================================================================================================

# BEGIN_SECTION:ADVICE_MAP_CIRCUIT_DESCRIPTION
# END_SECTION:ADVICE_MAP_CIRCUIT_DESCRIPTION

"#;
