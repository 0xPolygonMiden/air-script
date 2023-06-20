pub mod codegen;
mod config;
pub mod constants;
mod error;
mod exemption_points;
mod utils;
pub mod visitor;
mod writer;

pub use codegen::CodeGenerator;
pub use config::CodegenConfig;
pub use error::CodegenError;
use ir::AirIR;

// CODEGEN
// ================================================================================================

/// Given a [AirIR] generates code to evaluate the boundary and transition constraints in Masm.
pub fn code_gen(air: &AirIR) -> Result<String, CodegenError> {
    let codegen = CodeGenerator::new(air, CodegenConfig::default());
    codegen.generate()
}
