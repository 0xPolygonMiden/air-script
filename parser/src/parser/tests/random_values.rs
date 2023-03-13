use crate::ast::ConstraintType;

use super::{
    build_parse_test, Error, Expression::*, Identifier, IntegrityConstraint, IntegrityStmt::*,
    ParseError, RandBinding, RandomValues, Source, SourceSection, SourceSection::*,
};

// RANDOM VALUES
// ================================================================================================

#[test]
fn random_values_fixed_list() {
    let source = "
    random_values:
        rand: [15]";
    let expected = Source(vec![RandomValues(RandomValues::new(
        Identifier("rand".to_string()),
        15,
        vec![],
    ))]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn random_values_ident_vector() {
    let source = "
    random_values:
        rand: [a, b[12], c]";
    let expected = Source(vec![RandomValues(RandomValues::new(
        Identifier("rand".to_string()),
        14,
        vec![
            RandBinding::new(Identifier("a".to_string()), 1),
            RandBinding::new(Identifier("b".to_string()), 12),
            RandBinding::new(Identifier("c".to_string()), 1),
        ],
    ))]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn random_values_custom_name() {
    let source = "
    random_values:
        alphas: [14]";
    let expected = Source(vec![RandomValues(RandomValues::new(
        Identifier("alphas".to_string()),
        14,
        vec![],
    ))]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn random_values_empty_list_error() {
    let source = "
    random_values:
        rand: []";
    let error = Error::ParseError(ParseError::InvalidRandomValues(
        "Random Values section cannot be empty".to_string(),
    ));
    build_parse_test!(source).expect_error(error)
}

#[test]
fn random_values_multiple_declaration_error() {
    let source = "
    random_values:
        rand: [12]
        alphas: [a, b[2]]";
    let error = Error::ParseError(ParseError::InvalidRandomValues(
        "No more than one set of random values can be declared".to_string(),
    ));
    build_parse_test!(source).expect_error(error)
}

#[test]
fn random_values_index_access() {
    let source = "
    integrity_constraints:
        enf a + $alphas[1] = 0";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        ConstraintType::Inline(IntegrityConstraint::new(
            Add(
                Box::new(Elem(Identifier("a".to_string()))),
                Box::new(Rand(Identifier("alphas".to_string()), 1)),
            ),
            Const(0),
        )),
        None,
    )])]);
    build_parse_test!(source).expect_ast(expected);
}
