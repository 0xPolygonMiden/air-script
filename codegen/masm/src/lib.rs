pub mod codegen;
mod config;
pub mod constants;
pub mod visitor;
mod writer;

use codegen::{Codegen, CodegenError};
use config::CodegenConfig;
use ir::AirIR;

// CODEGEN
// ================================================================================================

/// Given a [AirIR] generates code to evaluate the boundary and transition constraints in Masm.
pub fn code_gen(air: &AirIR) -> Result<String, CodegenError> {
    let config = CodegenConfig {
        trace_len_address: constants::TRACE_LEN_ADDRESS,
        z_address: constants::Z_ADDRESS,
        ood_frame_address: constants::OOD_FRAME_ADDRESS,
        ood_aux_frame_address: constants::OOD_AUX_FRAME_ADDRESS,
        composition_coef_address: constants::COMPOSITION_COEF_ADDRESS,
        aux_rand_address: constants::AUX_RAND_ELEM_PTR,
        periodic_values_address: constants::PERIODIC_VALUES_ADDRESS,
        z_exp_address: constants::Z_EXP_ADDRESS,
    };
    let codegen = Codegen::new(air, config);
    codegen.generate()
}
