use super::{Identifier, ParseTest, Source, SourceSection};
use crate::ast::{ConstantBinding, ConstantValueExpr};

// CONSTANTS
// ================================================================================================

#[test]
fn constants_scalars() {
    let source = "
    const A = 1
    const B = 2";
    let expected = Source(vec![
        SourceSection::Constant(ConstantBinding::new(
            Identifier("A".to_string()),
            ConstantValueExpr::Scalar(1),
        )),
        SourceSection::Constant(ConstantBinding::new(
            Identifier("B".to_string()),
            ConstantValueExpr::Scalar(2),
        )),
    ]);
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn constants_vectors() {
    let source = "
    const A = [1, 2, 3, 4]
    const B = [5, 6, 7, 8]";
    let expected = Source(vec![
        SourceSection::Constant(ConstantBinding::new(
            Identifier("A".to_string()),
            ConstantValueExpr::Vector(vec![1, 2, 3, 4]),
        )),
        SourceSection::Constant(ConstantBinding::new(
            Identifier("B".to_string()),
            ConstantValueExpr::Vector(vec![5, 6, 7, 8]),
        )),
    ]);
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn constants_matrices() {
    let source = "
    const ABC = [[1, 2], [3, 4]]
    const XYZ = [[5, 6], [7, 8]]";
    let expected = Source(vec![
        SourceSection::Constant(ConstantBinding::new(
            Identifier("ABC".to_string()),
            ConstantValueExpr::Matrix(vec![vec![1, 2], vec![3, 4]]),
        )),
        SourceSection::Constant(ConstantBinding::new(
            Identifier("XYZ".to_string()),
            ConstantValueExpr::Matrix(vec![vec![5, 6], vec![7, 8]]),
        )),
    ]);
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn const_matrix_unequal_number_of_cols() {
    // This is invalid since the number of columns for the two rows are unequal. However this
    // validation happens at the IR level.
    let source = "
    const A = [[1, 2], [3, 4, 5]]";
    let expected = Source(vec![SourceSection::Constant(ConstantBinding::new(
        Identifier("A".to_string()),
        ConstantValueExpr::Matrix(vec![vec![1, 2], vec![3, 4, 5]]),
    ))]);
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn error_empty_constant_section() {
    let source = "
    const
    ";
    assert!(ParseTest::new().parse(source).is_err());
}

#[test]
fn error_empty_constant_declaration() {
    let source = "
    const A
    ";
    assert!(ParseTest::new().parse(source).is_err());
}

#[test]
fn err_lowercase_constant_name() {
    let source = "
    const Ab = [[1, 2], [3, 4]]
    const C = [[5, 6], [7, 8]]";
    ParseTest::new().expect_diagnostic(source, "constant identifiers must be uppercase");
}

#[test]
fn err_consts_with_non_int_values() {
    let source = "
        const A = a
        const B = 2";
    ParseTest::new().expect_unrecognized_token(source);
}

#[test]
fn err_const_vectors_with_non_int_values() {
    let source = "
        const A = [1, a]
        const B = [2, 4]";
    ParseTest::new().expect_unrecognized_token(source);
}

#[test]
fn err_vector_with_trailing_comma() {
    let source = "
    const A = [1, ]";
    ParseTest::new().expect_unrecognized_token(source);
}

#[test]
fn err_matrix_with_trailing_comma() {
    let source = "
    const A = [[1, 2], ]";
    ParseTest::new().expect_unrecognized_token(source);
}

#[test]
fn err_matrix_mixed_element_types() {
    let source = "
    const A = [1, [1, 2]]";
    ParseTest::new().expect_unrecognized_token(source);
}

#[test]
fn err_invalid_matrix_element() {
    let source = "
    const A = [[1, 2], [3, [4, 5]]]";
    ParseTest::new().expect_unrecognized_token(source);
}
