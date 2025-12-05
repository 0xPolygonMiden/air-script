/// Macros to generate Air tester structs and tests for both Plonky3 and Winterfell backends.
pub mod air_tester_macros;
/// Code generation for tests/**/*.air files.
pub mod codegen;
/// Miden VM auxiliary trace generator
pub mod miden_vm_aux_trace_generator;
/// Winterfell-specific traits
pub mod winterfell_traits;
