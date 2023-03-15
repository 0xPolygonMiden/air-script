use super::{build_parse_test, Identifier, IntegrityConstraint, Source, SourceSection};
use crate::ast::{ConstraintType, Expression::*, IntegrityStmt::*, NamedTraceAccess};

// SELECTORS
// ================================================================================================

#[test]
fn single_selector() {
    let source = "
    integrity_constraints:
        enf clk' = clk when n1";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        ConstraintType::Inline(IntegrityConstraint::new(
            // clk' = clk
            NamedTraceAccess(NamedTraceAccess::new(Identifier("clk".to_string()), 0, 1)),
            Elem(Identifier("clk".to_string())),
        )),
        // n1
        Some(Elem(Identifier("n1".to_string()))),
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn chained_selectors() {
    let source = "
    integrity_constraints:
        enf clk' = clk when (n1 & !n2) | !n3";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        ConstraintType::Inline(IntegrityConstraint::new(
            // clk' = clk
            NamedTraceAccess(NamedTraceAccess::new(Identifier("clk".to_string()), 0, 1)),
            Elem(Identifier("clk".to_string())),
        )),
        // (n1 & !n2) | !n3
        Some(Sub(
            Box::new(Add(
                Box::new(Mul(
                    Box::new(Elem(Identifier("n1".to_string()))),
                    Box::new(Sub(
                        Box::new(Const(1)),
                        Box::new(Elem(Identifier("n2".to_string()))),
                    )),
                )),
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(Elem(Identifier("n3".to_string()))),
                )),
            )),
            Box::new(Mul(
                Box::new(Mul(
                    Box::new(Elem(Identifier("n1".to_string()))),
                    Box::new(Sub(
                        Box::new(Const(1)),
                        Box::new(Elem(Identifier("n2".to_string()))),
                    )),
                )),
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(Elem(Identifier("n3".to_string()))),
                )),
            )),
        )),
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multiconstraint_selectors() {
    let source = "
    integrity_constraints:
        enf clk' = 0 when n1 & !n2
        match enf:
            clk' = clk when n1 & n2
            clk' = 1 when !n1 & !n2";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                // clk' = 0 when n1 & !n2
                NamedTraceAccess(NamedTraceAccess::new(Identifier("clk".to_string()), 0, 1)),
                Const(0),
            )),
            Some(Mul(
                Box::new(Elem(Identifier("n1".to_string()))),
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(Elem(Identifier("n2".to_string()))),
                )),
            )),
        ),
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                // clk' = clk when n1 & n2
                NamedTraceAccess(NamedTraceAccess::new(Identifier("clk".to_string()), 0, 1)),
                Elem(Identifier("clk".to_string())),
            )),
            Some(Mul(
                Box::new(Elem(Identifier("n1".to_string()))),
                Box::new(Elem(Identifier("n2".to_string()))),
            )),
        ),
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                // clk' = 1 when !n1 & !n2
                NamedTraceAccess(NamedTraceAccess::new(Identifier("clk".to_string()), 0, 1)),
                Const(1),
            )),
            Some(Mul(
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(Elem(Identifier("n1".to_string()))),
                )),
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(Elem(Identifier("n2".to_string()))),
                )),
            )),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}
