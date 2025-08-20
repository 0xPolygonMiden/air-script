use std::ops::Div;

use super::MasmVerifierParameters;
use crate::masm::{DOUBLE_WORD_SIZE, FIELD_EXTENSION_DEGREE};

/// Generates the MASM module of the STARK verifier for processing the out-of-domain (OOD)
/// evaluations.
pub fn generate_ood_frames_module(masm_verifier_parameters: &MasmVerifierParameters) -> String {
    let main_trace_width = masm_verifier_parameters.main_trace_width();
    let aux_trace_width = masm_verifier_parameters.aux_trace_width().unwrap_or(0);
    let num_constraints_composition_polys =
        masm_verifier_parameters.constraints_composition_trace_width();

    // we are loading and hashing extension field elements and hence we need to first compute
    // the number of extension field elements to process and thereafter convert this to
    // a number over base field elements
    let num_extension_field_elements =
        main_trace_width + aux_trace_width + num_constraints_composition_polys as u16;
    let num_base_field_elements = num_extension_field_elements * FIELD_EXTENSION_DEGREE as u16;

    // since we are loading two words per iteration, we need to divide by `DOUBLE_WORD_SIZE`
    let num_iterations = num_base_field_elements.div(DOUBLE_WORD_SIZE as u16);

    // we check double-word alignment
    debug_assert_eq!(
        num_base_field_elements % DOUBLE_WORD_SIZE as u16,
        0,
        "each of trace is expected to be double-word aligned"
    );

    OOD_FRAMES_MASM
        .to_string()
        .replace("{NUM_ITERATIONS_PROCESS_OOD_EVALS}", &num_iterations.to_string())
}

// TEMPLATES
// ================================================================================================

const OOD_FRAMES_MASM: &str = r#"
#! Processes the out-of-domain (OOD) evaluations of all committed polynomials.
#!
#! Takes as input an RPO hasher state and a pointer, and loads from the advice provider the OOD
#! evaluations and stores at memory region using pointer `ptr` while absorbing the evaluations
#! into the hasher state and simultaneously computing a random linear combination using Horner
#! evaluation.
#! 
#!
#! Inputs:  [R2, R1, C, ptr, acc1, acc0]
#! Outputs: [R2, R1, C, ptr, acc1`, acc0`]
export.process_row_ood_evaluations
    repeat.{NUM_ITERATIONS_PROCESS_OOD_EVALS}
        adv_pipe
        horner_eval_ext
        hperm
    end
end
"#;
