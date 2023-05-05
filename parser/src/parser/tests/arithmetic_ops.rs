use super::{
    AccessType, ConstraintExpr, Expression::*, Identifier, InlineConstraintExpr,
    IntegrityConstraint, IntegrityStmt::*, ParseTest, Source, SourceSection::*, SymbolAccess,
};

// EXPRESSIONS
// ================================================================================================

#[test]
fn single_addition() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf clk' + clk = 0";
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Add(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                        1,
                    ))),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
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

#[test]
fn multi_addition() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf clk' + clk + 2 = 0";
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Add(
                    Box::new(Add(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("clk".to_string()),
                            AccessType::Default,
                            1,
                        ))),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("clk".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                    )),
                    Box::new(Const(2)),
                ),
                Const(0),
            )),
            None,
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected)
}

#[test]
fn single_subtraction() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf clk' - clk = 0";
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Sub(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                        1,
                    ))),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                        0,
                    ))),
                ),
                Const(0),
            )),
            None,
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected)
}

#[test]
fn multi_subtraction() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf clk' - clk - 1 = 0";
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Sub(
                    Box::new(Sub(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("clk".to_string()),
                            AccessType::Default,
                            1,
                        ))),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("clk".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                    )),
                    Box::new(Const(1)),
                ),
                Const(0),
            )),
            None,
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected)
}

#[test]
fn single_multiplication() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf clk' * clk = 0";
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Mul(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                        1,
                    ))),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                        0,
                    ))),
                ),
                Const(0),
            )),
            None,
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected)
}

#[test]
fn multi_multiplication() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf clk' * clk * 2 = 0";
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Mul(
                    Box::new(Mul(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("clk".to_string()),
                            AccessType::Default,
                            1,
                        ))),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("clk".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                    )),
                    Box::new(Const(2)),
                ),
                Const(0),
            )),
            None,
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected)
}

#[test]
fn unit_with_parens() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf (2) + 1 = 3";
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Add(Box::new(Const(2)), Box::new(Const(1))),
                Const(3),
            )),
            None,
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected)
}

#[test]
fn ops_with_parens() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf (clk' + clk) * 2 = 4";
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Mul(
                    Box::new(Add(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("clk".to_string()),
                            AccessType::Default,
                            1,
                        ))),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("clk".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                    )),
                    Box::new(Const(2)),
                ),
                Const(4),
            )),
            None,
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected)
}

#[test]
fn const_exponentiation() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf clk'^2 = 1";
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Exp(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                        1,
                    ))),
                    Box::new(Const(2)),
                ),
                Const(1),
            )),
            None,
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected)
}

#[test]
fn non_const_exponentiation() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf clk'^(clk + 2) = 1";
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Exp(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                        1,
                    ))),
                    Box::new(Add(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("clk".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                        Box::new(Const(2)),
                    )),
                ),
                Const(1),
            )),
            None,
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected)
}

#[test]
fn err_ops_without_matching_closing_parens() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf (clk' + clk * 2 = 4";
    ParseTest::new().expect_unrecognized_token(source)
}

#[test]
fn err_closing_paren_without_opening_paren() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf clk' + clk) * 2 = 4";
    ParseTest::new().expect_unrecognized_token(source)
}

#[test]
fn multi_arithmetic_ops_same_precedence() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf clk' - clk - 2 + 1 = 0";
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Add(
                    Box::new(Sub(
                        Box::new(Sub(
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("clk".to_string()),
                                AccessType::Default,
                                1,
                            ))),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("clk".to_string()),
                                AccessType::Default,
                                0,
                            ))),
                        )),
                        Box::new(Const(2)),
                    )),
                    Box::new(Const(1)),
                ),
                Const(0),
            )),
            None,
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected)
}

#[test]
fn multi_arithmetic_ops_different_precedence() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf clk'^2 - clk * 2 - 1 = 0";
    // The precedence order of operations here is:
    // 1. Exponentiation
    // 2. Multiplication
    // 3. Addition/Subtraction
    // These operations are evaluated in the order of decreasing precedence.
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Sub(
                    Box::new(Sub(
                        Box::new(Exp(
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("clk".to_string()),
                                AccessType::Default,
                                1,
                            ))),
                            Box::new(Const(2)),
                        )),
                        Box::new(Mul(
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("clk".to_string()),
                                AccessType::Default,
                                0,
                            ))),
                            Box::new(Const(2)),
                        )),
                    )),
                    Box::new(Const(1)),
                ),
                Const(0),
            )),
            None,
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected)
}

#[test]
fn multi_arithmetic_ops_different_precedence_w_parens() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    integrity_constraints:
        enf clk' - clk^2 * (2 - 1) = 0";
    // The precedence order of operations here is:
    // 1. Parentheses
    // 2. Exp
    // 3. Multiplication
    // 4. Addition/Subtraction
    // These operations are evaluated in the order of decreasing precedence.
    let expected = Source(vec![IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Inline(InlineConstraintExpr::new(
                Sub(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                        1,
                    ))),
                    Box::new(Mul(
                        Box::new(Exp(
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("clk".to_string()),
                                AccessType::Default,
                                0,
                            ))),
                            Box::new(Const(2)),
                        )),
                        Box::new(Sub(Box::new(Const(2)), Box::new(Const(1)))),
                    )),
                ),
                Const(0),
            )),
            None,
            None,
        ),
    )])]);
    ParseTest::new().expect_ast(source, expected)
}
