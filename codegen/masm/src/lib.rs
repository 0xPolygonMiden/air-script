pub mod codegen;
mod config;
pub mod constants;
pub mod visitor;
mod writer;

pub use codegen::{CodeGenerator, CodegenError};
pub use config::CodegenConfig;
use ir::AirIR;

// CODEGEN
// ================================================================================================

/// Given a [AirIR] generates code to evaluate the boundary and transition constraints in Masm.
pub fn code_gen(air: &AirIR) -> Result<String, CodegenError> {
    let codegen = CodeGenerator::new(air, CodegenConfig::default());
    codegen.generate()
}
