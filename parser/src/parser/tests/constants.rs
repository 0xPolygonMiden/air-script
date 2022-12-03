use super::{build_parse_test, Identifier, Source, SourceSection};
use crate::{
    ast::constants::{Constant, ConstantType},
    error::{Error, ParseError},
};

// CONSTANTS
// ================================================================================================

#[test]
fn constants_scalars() {
    let source = "constants:
        A: 1
        B: 2";
    let expected = Source(vec![SourceSection::Constants(vec![
        Constant::new(Identifier("A".to_string()), ConstantType::Scalar(1)),
        Constant::new(Identifier("B".to_string()), ConstantType::Scalar(2)),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn constants_vectors() {
    let source = "constants:
        A: [1, 2, 3, 4]
        B: [5, 6, 7, 8]";
    let expected = Source(vec![SourceSection::Constants(vec![
        Constant::new(
            Identifier("A".to_string()),
            ConstantType::Vector(vec![1, 2, 3, 4]),
        ),
        Constant::new(
            Identifier("B".to_string()),
            ConstantType::Vector(vec![5, 6, 7, 8]),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn constants_matrices() {
    let source = "constants:
        ABC: [[1, 2], [3, 4]]
        XYZ: [[5, 6], [7, 8]]";
    let expected = Source(vec![SourceSection::Constants(vec![
        Constant::new(
            Identifier("ABC".to_string()),
            ConstantType::Matrix(vec![vec![1, 2], vec![3, 4]]),
        ),
        Constant::new(
            Identifier("XYZ".to_string()),
            ConstantType::Matrix(vec![vec![5, 6], vec![7, 8]]),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn const_matrix_unequal_number_of_cols() {
    // This is invalid since the number of columns for the two rows are unequal. However this
    // validation happens at the IR level.
    let source = "constants:
    A: [[1, 2], [3, 4, 5]]";
    let expected = Source(vec![SourceSection::Constants(vec![Constant::new(
        Identifier("A".to_string()),
        ConstantType::Matrix(vec![vec![1, 2], vec![3, 4, 5]]),
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn error_empty_constants_section() {
    let source = "
    constants:
    ";
    assert!(build_parse_test!(source).parse().is_err());
}

#[test]
fn err_lowercase_constant_name() {
    let source = "constants:
    Ab: [[1, 2], [3, 4]]
    C: [[5, 6], [7, 8]]";
    let error = Error::ParseError(ParseError::InvalidConst(
        "The constant name should be uppercase: Ab".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

#[test]
fn err_consts_with_non_int_values() {
    let source = "constants:
        A: a
        B: 2";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn err_const_vectors_with_non_int_values() {
    let source = "constants:
        A: [1, a]
        B: [2, 4]";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn err_vector_with_trailing_comma() {
    let source = "constants:
    A: [1, ]";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn err_matrix_with_trailing_comma() {
    let source = "constants:
    A: [[1, 2], ]";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn err_matrix_mixed_element_types() {
    let source = "constants:
    A: [1, [1, 2]]";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn err_invalid_matrix_element() {
    let source = "constants:
    A: [[1, 2], [3, [4, 5]]]";
    build_parse_test!(source).expect_unrecognized_token();
}
