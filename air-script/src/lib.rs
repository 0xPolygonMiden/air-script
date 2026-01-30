pub use air_codegen_winter::CodeGenerator as WinterfellCodeGenerator;
pub use air_ir::{Air, CompileError, compile};
pub use air_parser::{parse, parse_file, transforms};

/// Miden VM auxiliary trace generator
pub mod miden_vm_aux_trace_generator;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
mod tests;
