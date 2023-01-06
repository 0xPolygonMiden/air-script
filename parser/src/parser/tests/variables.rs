use super::build_parse_test;

// VARIABLES INVALID USAGE
// ================================================================================================

#[test]
fn err_vector_defined_outside_boundary_or_transition_constraints() {
    let source = "
        const A = 1
        let a = 0";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn err_vector_variable_with_trailing_comma() {
    let source = "
    transition_constraints:
        let a = [1, ]";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn err_matrix_variable_with_trailing_comma() {
    let source = "
    transition_constraints:
        let a = [[1, 2], ]";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn err_matrix_variable_mixed_element_types() {
    let source = "transition_constraints:
    let a = [[1, 2], 1]";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn err_invalid_matrix_element() {
    let source = "transition_constraints:
    let a = [[1, 2], [3, [4, 5]]]";
    build_parse_test!(source).expect_unrecognized_token();
}
