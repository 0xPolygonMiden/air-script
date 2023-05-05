use super::{
    AccessType, Expression::*, Identifier, InlineConstraintExpr, IntegrityConstraint,
    IntegrityStmt::*, ParseTest, RandBinding, RandomValues, Source, SourceSection,
    SourceSection::*, SymbolAccess,
};
use crate::ast::ConstraintExpr;

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
    ParseTest::new().expect_ast(source, expected);
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
    ParseTest::new().expect_ast(source, expected);
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
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn random_values_empty_list_error() {
    let source = "
    random_values:
        rand: []";
    ParseTest::new().expect_diagnostic(source, "random values cannot be empty");
}

#[test]
fn random_values_multiple_declaration_error() {
    let source = "
    random_values:
        rand: [12]
        alphas: [a, b[2]]";
    ParseTest::new().expect_diagnostic(source, "only one declaration may appear in random_values");
}

#[test]
fn random_values_index_access() {
    let source = "
    integrity_constraints:
        enf a + $alphas[1] = 0";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
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
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected);
}
