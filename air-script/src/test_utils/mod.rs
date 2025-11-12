/// Macros to generate Air tester structs and tests for both Plonky3 and Winterfell backends.
pub mod air_tester_macros;
/// Code generation for tests/**/*.air files.
pub mod codegen;
/// Plonky3-specific traits, to be moved to 0xMiden/Plonky3 once stabilized.
pub mod plonky3_traits;
/// Winterfell-specific traits
pub mod winterfell_traits;
