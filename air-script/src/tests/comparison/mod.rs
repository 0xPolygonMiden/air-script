//! Cross-backend comparison tests.
//!
//! This module contains tests that verify Winterfell and Plonky3 backends
//! produce equivalent constraint evaluations for the same AIR and trace data.

mod binary;
mod bitwise;
mod computed_indices_complex;
mod computed_indices_simple;
mod constant_in_range;
mod constants;
mod constraint_comprehension;
mod cross_module_constants;
mod evaluators;
mod evaluators_nested_slice_call;
mod evaluators_slice;
mod fibonacci;
mod indexed_trace_access;
