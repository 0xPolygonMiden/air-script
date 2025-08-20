use crate::masm::MasmVerifierParameters;

/// Generates the main MASM module of the STARK verifier.
pub fn generate_verifier_module(masm_verifier_parameters: &MasmVerifierParameters) -> String {
    let is_aux_trace = masm_verifier_parameters.aux_trace_width().is_some() as u8;
    let trace_info = build_trace_info(masm_verifier_parameters);

    VERIFIER_MASM
        .to_string()
        .replace("IS_AUX_TRACE_VALUE", &is_aux_trace.to_string())
        .replace("TRACE_INFO_VALUE", &trace_info)
}

// HELPERS
// ================================================================================================

/// Builds the trace info constant.
fn build_trace_info(masm_verifier_parameters: &MasmVerifierParameters) -> String {
    let main_segment_width = masm_verifier_parameters.main_trace_width as u8;
    let (num_aux_segments, aux_segment_width): (u8, u8) =
        match masm_verifier_parameters.aux_trace_width {
            Some(aux_seg_width) => (1, aux_seg_width as u8),
            None => (0, 0),
        };
    let num_aux_randomness = masm_verifier_parameters.num_auxiliary_randomness() as u8;

    "0x".to_string()
        + &format!("{main_segment_width:02x}",).to_string()
        + &format!("{num_aux_segments:02x}",).to_string()
        + &format!("{aux_segment_width:02x}",).to_string()
        + &format!("{num_aux_randomness:02x}",).to_string()
}

// TEMPLATES
// ================================================================================================

const VERIFIER_MASM: &str = r#"
use.std::crypto::hashes::rpo

use.std::sys::vm::deep_queries
use.std::sys::vm::constraints_eval
use.std::sys::vm::ood_frames
use.std::sys::vm::public_inputs

use.std::crypto::stark::verifier

# Indicates the existence of auxiliary trace segment.
const.IS_AUX_TRACE=IS_AUX_TRACE_VALUE

# A constant encoding the main segment width, the number of auxiliary segments (either 0 or 1),
# width of the auxiliary segment if it exists and the number of auxiliary randomness used by it
const.TRACE_INFO=TRACE_INFO_VALUE

#! Verifies STARK proof.
#!
#! Inputs:  [log(trace_length), num_queries, grinding]
#! Outputs: []
export.verify_proof
    # --- Get constants -------------------------------------------------------

    # Flag indicating the existence of auxiliary trace
    push.IS_AUX_TRACE movdn.3
    # => [log(trace_length), num_queries, grinding, is_aux_trace]

    # Number of fixed length public inputs
    exec.public_inputs::get_num_fixed_len_public_inputs movdn.3
    # => [log(trace_length), num_queries, grinding, num_fixed_len_pi, is_aux_trace]

    # Trace info as one field element
    push.TRACE_INFO movdn.3
    # => [log(trace_length), num_queries, grinding, trace_info, num_fixed_len_pi, is_aux_trace]

    # Number of constraints
    exec.constraints_eval::get_num_constraints movdn.3
    # => [log(trace_length), num_queries, grinding, num_constraints, trace_info, num_fixed_len_pi, is_aux_trace]

    # --- Load the digests of all dynamically invoked procedures --------------

    procref.deep_queries::compute_deep_composition_polynomial_queries
    procref.constraints_eval::execute_constraint_evaluation_check
    procref.ood_frames::process_row_ood_evaluations
    procref.public_inputs::process_public_inputs
    # =>[D3, D2, D1, D0, log(trace_length), num_queries, grinding, num_constraints, trace_info, num_fixed_len_pi, is_aux_trace]

    # --- Call the core verification procedure from `stdlib` ------------------

    exec.verifier::verify
    # => [...]
end
"#;
