// EXPORTS
// ================================================================================================

/// AirScript parse method to generate an AST from AirScript source files
pub use air_parser::parse;

/// AirScript intermediate representation
pub use air_ir::Air;

/// Code generation targeting Rust for the Winterfell prover
pub use codegen_winter::CodeGenerator;
