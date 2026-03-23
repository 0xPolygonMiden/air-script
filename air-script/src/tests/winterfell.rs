use expect_test::expect_file;

use crate::test_utils::codegen::{Target, Test};

#[test]
fn binary() {
    let generated_air = Test::new("src/tests/binary/binary.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["binary/binary.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn bitwise() {
    let generated_air = Test::new("src/tests/bitwise/bitwise.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["bitwise/bitwise.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn buses_complex() {
    let generated_air = Test::new("src/tests/buses/buses_complex.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["buses/buses_complex.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn buses_simple() {
    let generated_air = Test::new("src/tests/buses/buses_simple.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["buses/buses_simple.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn buses_simple_with_evaluators() {
    let generated_air = Test::new("src/tests/buses/buses_simple_with_evaluators.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["buses/buses_simple.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn buses_sum_form() {
    let generated_air = Test::new("src/tests/buses/buses_sum_form.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["buses/buses_sum_form.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn buses_varlen_boundary_both() {
    let generated_air = Test::new("src/tests/buses/buses_varlen_boundary_both.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["buses/buses_varlen_boundary_both.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn buses_varlen_boundary_first() {
    let generated_air = Test::new("src/tests/buses/buses_varlen_boundary_first.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["buses/buses_varlen_boundary_first.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn buses_varlen_boundary_last() {
    let generated_air = Test::new("src/tests/buses/buses_varlen_boundary_last.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["buses/buses_varlen_boundary_last.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn comprehension_periodic_binding() {
    // Test that comprehension bindings over periodic columns are typed as Local, not PeriodicColumn
    // This pattern is used when iterating over a vector containing periodic column references
    let generated_air = Test::new(
        "src/tests/comprehension_periodic_binding/comprehension_periodic_binding.air".to_string(),
    )
    .transpile(Target::Winterfell)
    .unwrap();

    let expected = expect_file!["comprehension_periodic_binding/comprehension_periodic_binding.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn computed_indices_complex() {
    let generated_air =
        Test::new("src/tests/computed_indices/computed_indices_complex.air".to_string())
            .transpile(Target::Winterfell)
            .unwrap();

    let expected = expect_file!["computed_indices/computed_indices_complex.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn computed_indices_simple() {
    let generated_air =
        Test::new("src/tests/computed_indices/computed_indices_simple.air".to_string())
            .transpile(Target::Winterfell)
            .unwrap();

    let expected = expect_file!["computed_indices/computed_indices_simple.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn constant_in_range() {
    let generated_air = Test::new("src/tests/constant_in_range/constant_in_range.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["constant_in_range/constant_in_range.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn constants() {
    let generated_air = Test::new("src/tests/constants/constants.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["constants/constants.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn constraint_comprehension() {
    let generated_air =
        Test::new("src/tests/constraint_comprehension/constraint_comprehension.air".to_string())
            .transpile(Target::Winterfell)
            .unwrap();

    let expected = expect_file!["constraint_comprehension/constraint_comprehension.rs"];
    expected.assert_eq(&generated_air);

    let generated_air =
        Test::new("src/tests/constraint_comprehension/cc_with_evaluators.air".to_string())
            .transpile(Target::Winterfell)
            .unwrap();

    let expected = expect_file!["constraint_comprehension/constraint_comprehension.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn cross_module_constants() {
    // Test that constants used in comprehension iterables work across module boundaries
    let generated_air =
        Test::new("src/tests/cross_module_constants/cross_module_constants.air".to_string())
            .transpile(Target::Winterfell)
            .unwrap();

    let expected = expect_file!["cross_module_constants/cross_mod_constants.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn evaluators_nested_slice_call() {
    let generated_air =
        Test::new("src/tests/evaluators/evaluators_nested_slice_call.air".to_string())
            .transpile(Target::Winterfell)
            .unwrap();

    let expected = expect_file!["evaluators/evaluators_nested_slice_call.rs"];
    expected.assert_eq(&generated_air);
}

// TODO: add support for nested slicing in general expressions.
//
// #[test]
// fn evaluators_slice_slicing() {
//     let generated_air =
// Test::new("src/tests/evaluators/evaluators_slice_slicing.air".to_string())
//         .transpile(Target::Winterfell)
//         .unwrap();
//
//     let expected = expect_file!["evaluators/evaluators_slice_slicing.rs"];
//     expected.assert_eq(&generated_air);
// }

#[test]
fn evaluators_slice() {
    let generated_air = Test::new("src/tests/evaluators/evaluators_slice.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["evaluators/evaluators_slice.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn evaluators() {
    let generated_air = Test::new("src/tests/evaluators/evaluators.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["evaluators/evaluators.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn fibonacci() {
    let generated_air = Test::new("src/tests/fibonacci/fibonacci.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["fibonacci/fibonacci.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn functions_complex() {
    let generated_air = Test::new("src/tests/functions/functions_complex.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["functions/functions_complex.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn functions_simple() {
    let generated_air = Test::new("src/tests/functions/functions_simple.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["functions/functions_simple.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn functions_simple_inlined() {
    // make sure that the constraints generated using inlined functions are the same as the ones
    // generated using regular functions
    let generated_air = Test::new("src/tests/functions/inlined_functions_simple.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["functions/functions_simple.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn indexed_trace_access() {
    let generated_air =
        Test::new("src/tests/indexed_trace_access/indexed_trace_access.air".to_string())
            .transpile(Target::Winterfell)
            .unwrap();

    let expected = expect_file!["indexed_trace_access/indexed_trace_access.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn list_comprehension() {
    let generated_air =
        Test::new("src/tests/list_comprehension/list_comprehension.air".to_string())
            .transpile(Target::Winterfell)
            .unwrap();

    let expected = expect_file!["list_comprehension/list_comprehension.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn list_comprehension_nested() {
    let generated_air =
        Test::new("src/tests/list_comprehension/list_comprehension_nested.air".to_string())
            .transpile(Target::Winterfell)
            .unwrap();

    let expected = expect_file!["list_comprehension/list_comprehension_nested.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn list_folding() {
    let generated_air = Test::new("src/tests/list_folding/list_folding.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["list_folding/list_folding.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn periodic_columns() {
    let generated_air = Test::new("src/tests/periodic_columns/periodic_columns.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["periodic_columns/periodic_columns.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn pub_inputs() {
    let generated_air = Test::new("src/tests/pub_inputs/pub_inputs.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["pub_inputs/pub_inputs.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn selectors() {
    let generated_air = Test::new("src/tests/selectors/selectors.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["selectors/selectors.rs"];
    expected.assert_eq(&generated_air);

    let generated_air = Test::new("src/tests/selectors/selectors_with_evaluators.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["selectors/selectors_with_evaluators.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn selectors_combine_simple() {
    let generated_air = Test::new("src/tests/selectors/selectors_combine_simple.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["selectors/selectors_combine_simple.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn selectors_combine_complex() {
    let generated_air = Test::new("src/tests/selectors/selectors_combine_complex.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["selectors/selectors_combine_complex.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn selectors_combine_with_list_comprehensions() {
    let generated_air =
        Test::new("src/tests/selectors/selectors_combine_with_list_comprehensions.air".to_string())
            .transpile(Target::Winterfell)
            .unwrap();

    let expected = expect_file!["selectors/selectors_combine_with_list_comprehensions.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn system() {
    let generated_air = Test::new("src/tests/system/system.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["system/system.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn trace_col_groups() {
    let generated_air = Test::new("src/tests/trace_col_groups/trace_col_groups.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["trace_col_groups/trace_col_groups.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn variables() {
    let generated_air = Test::new("src/tests/variables/variables.air".to_string())
        .transpile(Target::Winterfell)
        .unwrap();

    let expected = expect_file!["variables/variables.rs"];
    expected.assert_eq(&generated_air);
}
