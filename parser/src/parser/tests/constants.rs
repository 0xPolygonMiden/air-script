use crate::ast::constants::{Constant, ConstantType};

use super::{build_parse_test, Identifier, Source, SourceSection};

// CONSTANTS
// ================================================================================================

#[test]
fn constants_scalars() {
    let source = "constants:
        a: 1
        b: 2";
    let expected = Source(vec![SourceSection::Constants(vec![
        Constant::new(Identifier("a".to_string()), ConstantType::Scalar(1)),
        Constant::new(Identifier("b".to_string()), ConstantType::Scalar(2)),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn constants_vectors() {
    let source = "constants:
        a: [1, 2, 3, 4]
        b: [5, 6, 7, 8]";
    let expected = Source(vec![SourceSection::Constants(vec![
        Constant::new(
            Identifier("a".to_string()),
            ConstantType::Vector(vec![1, 2, 3, 4]),
        ),
        Constant::new(
            Identifier("b".to_string()),
            ConstantType::Vector(vec![5, 6, 7, 8]),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn constants_matrices() {
    let source = "constants:
        a: [[1, 2], [3, 4]]
        b: [[5, 6], [7, 8]]";
    let expected = Source(vec![SourceSection::Constants(vec![
        Constant::new(
            Identifier("a".to_string()),
            ConstantType::Matrix(vec![vec![1, 2], vec![3, 4]]),
        ),
        Constant::new(
            Identifier("b".to_string()),
            ConstantType::Matrix(vec![vec![5, 6], vec![7, 8]]),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn error_empty_constants_section() {
    let source = "
    constants:
    ";
    assert!(build_parse_test!(source).parse().is_err());
}
