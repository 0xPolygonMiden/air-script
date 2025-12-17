/// Macros to generate Air tester structs and tests for both Plonky3 and Winterfell backends.
pub mod air_tester_macros;
/// Code generation for tests/**/*.air files.
pub mod codegen;
/// Conversion utilities for test inputs (both public inputs and variable-length public inputs).
pub mod pub_inputs_conversion_utils;
/// Winterfell-specific traits
pub mod winterfell_traits;
