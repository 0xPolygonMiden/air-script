use super::{build_parse_test, Identifier, IntegrityConstraint, Source, SourceSection};
use crate::ast::{AccessType, ConstraintType, Expression::*, IntegrityStmt::*, SymbolAccess};

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
            SymbolAccess(SymbolAccess::new(
                Identifier("clk".to_string()),
                AccessType::Default,
                1,
            )),
            SymbolAccess(SymbolAccess::new(
                Identifier("clk".to_string()),
                AccessType::Default,
                0,
            )),
        )),
        // n1
        Some(SymbolAccess(SymbolAccess::new(
            Identifier("n1".to_string()),
            AccessType::Default,
            0,
        ))),
        None,
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
            SymbolAccess(SymbolAccess::new(
                Identifier("clk".to_string()),
                AccessType::Default,
                1,
            )),
            SymbolAccess(SymbolAccess::new(
                Identifier("clk".to_string()),
                AccessType::Default,
                0,
            )),
        )),
        // (n1 & !n2) | !n3
        Some(Sub(
            Box::new(Add(
                Box::new(Mul(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("n1".to_string()),
                        AccessType::Default,
                        0,
                    ))),
                    Box::new(Sub(
                        Box::new(Const(1)),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("n2".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                    )),
                )),
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("n3".to_string()),
                        AccessType::Default,
                        0,
                    ))),
                )),
            )),
            Box::new(Mul(
                Box::new(Mul(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("n1".to_string()),
                        AccessType::Default,
                        0,
                    ))),
                    Box::new(Sub(
                        Box::new(Const(1)),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("n2".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                    )),
                )),
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("n3".to_string()),
                        AccessType::Default,
                        0,
                    ))),
                )),
            )),
        )),
        None,
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
                SymbolAccess(SymbolAccess::new(
                    Identifier("clk".to_string()),
                    AccessType::Default,
                    1,
                )),
                Const(0),
            )),
            Some(Mul(
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("n1".to_string()),
                    AccessType::Default,
                    0,
                ))),
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("n2".to_string()),
                        AccessType::Default,
                        0,
                    ))),
                )),
            )),
            None,
        ),
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                // clk' = clk when n1 & n2
                SymbolAccess(SymbolAccess::new(
                    Identifier("clk".to_string()),
                    AccessType::Default,
                    1,
                )),
                SymbolAccess(SymbolAccess::new(
                    Identifier("clk".to_string()),
                    AccessType::Default,
                    0,
                )),
            )),
            Some(Mul(
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("n1".to_string()),
                    AccessType::Default,
                    0,
                ))),
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("n2".to_string()),
                    AccessType::Default,
                    0,
                ))),
            )),
            None,
        ),
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                // clk' = 1 when !n1 & !n2
                SymbolAccess(SymbolAccess::new(
                    Identifier("clk".to_string()),
                    AccessType::Default,
                    1,
                )),
                Const(1),
            )),
            Some(Mul(
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("n1".to_string()),
                        AccessType::Default,
                        0,
                    ))),
                )),
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("n2".to_string()),
                        AccessType::Default,
                        0,
                    ))),
                )),
            )),
            None,
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}
