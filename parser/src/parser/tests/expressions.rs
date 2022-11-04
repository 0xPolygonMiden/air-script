use super::{
    build_parse_test, Identifier, Source, SourceSection, TransitionConstraint, TransitionExpr,
};

// EXPRESSIONS
// ================================================================================================

#[test]
fn single_addition() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' + clk = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Add(
                Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
            ),
            TransitionExpr::Const(0),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multi_addition() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' + clk + 2 = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Add(
                Box::new(TransitionExpr::Add(
                    Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                    Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
                )),
                Box::new(TransitionExpr::Const(2)),
            ),
            TransitionExpr::Const(0),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn single_subtraction() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' - clk = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Sub(
                Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
            ),
            TransitionExpr::Const(0),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multi_subtraction() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' - clk - 1 = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Sub(
                Box::new(TransitionExpr::Sub(
                    Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                    Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
                )),
                Box::new(TransitionExpr::Const(1)),
            ),
            TransitionExpr::Const(0),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn single_multiplication() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' * clk = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Mul(
                Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
            ),
            TransitionExpr::Const(0),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multi_multiplication() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' * clk * 2 = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Mul(
                Box::new(TransitionExpr::Mul(
                    Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                    Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
                )),
                Box::new(TransitionExpr::Const(2)),
            ),
            TransitionExpr::Const(0),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn unit_with_parens() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf (2) + 1 = 3";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Add(
                Box::new(TransitionExpr::Const(2)),
                Box::new(TransitionExpr::Const(1)),
            ),
            TransitionExpr::Const(3),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ops_with_parens() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf (clk' + clk) * 2 = 4";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Mul(
                Box::new(TransitionExpr::Add(
                    Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                    Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
                )),
                Box::new(TransitionExpr::Const(2)),
            ),
            TransitionExpr::Const(4),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn single_exponentiation() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk'^2 = 1";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Exp(
                Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                2,
            ),
            TransitionExpr::Const(1),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn err_ops_without_matching_closing_parens() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf (clk' + clk * 2 = 4";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn err_closing_paren_without_opening_paren() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' + clk) * 2 = 4";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn non_const_exp_fail() {
    // Should fail if the exponent is not a constant
    let source = "
    transition_constraints:
        enf clk'^a = 1";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn multi_arithmetic_ops_same_precedence() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' - clk - 2 + 1 = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Add(
                Box::new(TransitionExpr::Sub(
                    Box::new(TransitionExpr::Sub(
                        Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                        Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
                    )),
                    Box::new(TransitionExpr::Const(2)),
                )),
                Box::new(TransitionExpr::Const(1)),
            ),
            TransitionExpr::Const(0),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multi_arithmetic_ops_different_precedence() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk'^2 - clk * 2 - 1 = 0";
    // The precedence order of operations here is:
    // 1. Exponentiation
    // 2. Multiplication
    // 3. Addition/Subtraction
    // These operations are evaluated in the order of decreasing precedence.
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Sub(
                Box::new(TransitionExpr::Sub(
                    Box::new(TransitionExpr::Exp(
                        Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                        2,
                    )),
                    Box::new(TransitionExpr::Mul(
                        Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
                        Box::new(TransitionExpr::Const(2)),
                    )),
                )),
                Box::new(TransitionExpr::Const(1)),
            ),
            TransitionExpr::Const(0),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multi_arithmetic_ops_different_precedence_w_parens() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' - clk^2 * (2 - 1) = 0";
    // The precedence order of operations here is:
    // 1. Parentheses
    // 2. Exp
    // 3. Multiplication
    // 4. Addition/Subtraction
    // These operations are evaluated in the order of decreasing precedence.
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Sub(
                Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                Box::new(TransitionExpr::Mul(
                    Box::new(TransitionExpr::Exp(
                        Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
                        2,
                    )),
                    Box::new(TransitionExpr::Sub(
                        Box::new(TransitionExpr::Const(2)),
                        Box::new(TransitionExpr::Const(1)),
                    )),
                )),
            ),
            TransitionExpr::Const(0),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}
