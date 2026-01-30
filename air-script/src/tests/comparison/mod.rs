//! Cross-backend comparison tests.
//!
//! This module contains tests that verify Winterfell and Plonky3 backends
//! produce equivalent constraint evaluations for the same AIR and trace data.

mod binary;
mod bitwise;
mod cc_with_evaluators;
mod comprehension_periodic_binding;
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
mod functions_complex;
mod functions_simple;
mod indexed_trace_access;
mod inlined_functions_simple;
mod list_comprehension;
mod list_comprehension_nested;
mod list_folding;
mod periodic_columns;
mod pub_inputs;
mod selectors;
mod selectors_combine_simple;
mod selectors_combine_with_list_comprehensions;
mod selectors_with_evaluators;
mod system;
mod trace_col_groups;
mod variables;
