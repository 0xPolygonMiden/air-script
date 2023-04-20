use super::{
    build_parse_test, AccessType, Error, Expression::*, Identifier, IntegrityConstraint,
    IntegrityStmt::*, ParseError, RandBinding, RandomValues, Source, SourceSection,
    SourceSection::*, SymbolAccess,
};
use crate::ast::ConstraintType;

// RANDOM VALUES
// ================================================================================================

#[test]
fn random_values_fixed_list() {
    let source = "
    random_values:
        rand: [15]";
    let expected = Source(vec![RandomValues(RandomValues::new(
        15,
        vec![RandBinding::new(Identifier("$rand".to_string()), 15)],
    ))]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn random_values_ident_vector() {
    let source = "
    random_values:
        rand: [a, b[12], c]";
    let expected = Source(vec![RandomValues(RandomValues::new(
        14,
        vec![
            RandBinding::new(Identifier("$rand".to_string()), 14),
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
        14,
        vec![RandBinding::new(Identifier("$alphas".to_string()), 14)],
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
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("a".to_string()),
                    AccessType::Default,
                    0,
                ))),
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("$alphas".to_string()),
                    AccessType::Vector(1),
                    0,
                ))),
            ),
            Const(0),
        )),
        None,
    )])]);
    build_parse_test!(source).expect_ast(expected);
}
