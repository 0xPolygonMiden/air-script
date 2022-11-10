// EXPORTS
// ================================================================================================

/// AirScript parse method to generate an AST from AirScript source files
pub use parser::parse;

/// AirScript intermediate representation
pub use ir::AirIR;

/// Code generation targeting Rust for the Winterfell prover
pub use codegen_winter::CodeGenerator;
