use super::{build_parse_test, Identifier, IntegrityConstraint, Source, SourceSection};
use crate::ast::{
    AccessType, BindingAccess, ConstraintType, Expression::*, IntegrityStmt::*, TraceBindingAccess,
    TraceBindingAccessSize,
};

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
            TraceBindingAccess(TraceBindingAccess::new(
                Identifier("clk".to_string()),
                0,
                TraceBindingAccessSize::Full,
                1,
            )),
            BindingAccess(BindingAccess::new(
                Identifier("clk".to_string()),
                AccessType::Default,
            )),
        )),
        // n1
        Some(BindingAccess(BindingAccess::new(
            Identifier("n1".to_string()),
            AccessType::Default,
        ))),
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
            TraceBindingAccess(TraceBindingAccess::new(
                Identifier("clk".to_string()),
                0,
                TraceBindingAccessSize::Full,
                1,
            )),
            BindingAccess(BindingAccess::new(
                Identifier("clk".to_string()),
                AccessType::Default,
            )),
        )),
        // (n1 & !n2) | !n3
        Some(Sub(
            Box::new(Add(
                Box::new(Mul(
                    Box::new(BindingAccess(BindingAccess::new(
                        Identifier("n1".to_string()),
                        AccessType::Default,
                    ))),
                    Box::new(Sub(
                        Box::new(Const(1)),
                        Box::new(BindingAccess(BindingAccess::new(
                            Identifier("n2".to_string()),
                            AccessType::Default,
                        ))),
                    )),
                )),
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(BindingAccess(BindingAccess::new(
                        Identifier("n3".to_string()),
                        AccessType::Default,
                    ))),
                )),
            )),
            Box::new(Mul(
                Box::new(Mul(
                    Box::new(BindingAccess(BindingAccess::new(
                        Identifier("n1".to_string()),
                        AccessType::Default,
                    ))),
                    Box::new(Sub(
                        Box::new(Const(1)),
                        Box::new(BindingAccess(BindingAccess::new(
                            Identifier("n2".to_string()),
                            AccessType::Default,
                        ))),
                    )),
                )),
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(BindingAccess(BindingAccess::new(
                        Identifier("n3".to_string()),
                        AccessType::Default,
                    ))),
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
                TraceBindingAccess(TraceBindingAccess::new(
                    Identifier("clk".to_string()),
                    0,
                    TraceBindingAccessSize::Full,
                    1,
                )),
                Const(0),
            )),
            Some(Mul(
                Box::new(BindingAccess(BindingAccess::new(
                    Identifier("n1".to_string()),
                    AccessType::Default,
                ))),
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(BindingAccess(BindingAccess::new(
                        Identifier("n2".to_string()),
                        AccessType::Default,
                    ))),
                )),
            )),
        ),
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                // clk' = clk when n1 & n2
                TraceBindingAccess(TraceBindingAccess::new(
                    Identifier("clk".to_string()),
                    0,
                    TraceBindingAccessSize::Full,
                    1,
                )),
                BindingAccess(BindingAccess::new(
                    Identifier("clk".to_string()),
                    AccessType::Default,
                )),
            )),
            Some(Mul(
                Box::new(BindingAccess(BindingAccess::new(
                    Identifier("n1".to_string()),
                    AccessType::Default,
                ))),
                Box::new(BindingAccess(BindingAccess::new(
                    Identifier("n2".to_string()),
                    AccessType::Default,
                ))),
            )),
        ),
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                // clk' = 1 when !n1 & !n2
                TraceBindingAccess(TraceBindingAccess::new(
                    Identifier("clk".to_string()),
                    0,
                    TraceBindingAccessSize::Full,
                    1,
                )),
                Const(1),
            )),
            Some(Mul(
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(BindingAccess(BindingAccess::new(
                        Identifier("n1".to_string()),
                        AccessType::Default,
                    ))),
                )),
                Box::new(Sub(
                    Box::new(Const(1)),
                    Box::new(BindingAccess(BindingAccess::new(
                        Identifier("n2".to_string()),
                        AccessType::Default,
                    ))),
                )),
            )),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}
