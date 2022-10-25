use super::{
    build_parse_test, Expr, Identifier, Source, SourceSection, TransitionConstraint,
    TransitionConstraints,
};

// EXPRESSIONS
// ================================================================================================

#[test]
fn single_addition() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' + clk = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                Expr::Add(
                    Box::new(Expr::Next(Identifier("clk".to_string()))),
                    Box::new(Expr::Variable(Identifier("clk".to_string()))),
                ),
                Expr::Constant(0),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multi_addition() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' + clk + 2 = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                Expr::Add(
                    Box::new(Expr::Add(
                        Box::new(Expr::Next(Identifier("clk".to_string()))),
                        Box::new(Expr::Variable(Identifier("clk".to_string()))),
                    )),
                    Box::new(Expr::Constant(2)),
                ),
                Expr::Constant(0),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn single_subtraction() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' - clk = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                Expr::Subtract(
                    Box::new(Expr::Next(Identifier("clk".to_string()))),
                    Box::new(Expr::Variable(Identifier("clk".to_string()))),
                ),
                Expr::Constant(0),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multi_subtraction() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' - clk - 1 = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                Expr::Subtract(
                    Box::new(Expr::Subtract(
                        Box::new(Expr::Next(Identifier("clk".to_string()))),
                        Box::new(Expr::Variable(Identifier("clk".to_string()))),
                    )),
                    Box::new(Expr::Constant(1)),
                ),
                Expr::Constant(0),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn single_multiplication() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' * clk = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                Expr::Multiply(
                    Box::new(Expr::Next(Identifier("clk".to_string()))),
                    Box::new(Expr::Variable(Identifier("clk".to_string()))),
                ),
                Expr::Constant(0),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multi_multiplication() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' * clk * 2 = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                Expr::Multiply(
                    Box::new(Expr::Multiply(
                        Box::new(Expr::Next(Identifier("clk".to_string()))),
                        Box::new(Expr::Variable(Identifier("clk".to_string()))),
                    )),
                    Box::new(Expr::Constant(2)),
                ),
                Expr::Constant(0),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multi_arithmetic_ops_same_precedence() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' - clk - 2 + 1 = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                Expr::Add(
                    Box::new(Expr::Subtract(
                        Box::new(Expr::Subtract(
                            Box::new(Expr::Next(Identifier("clk".to_string()))),
                            Box::new(Expr::Variable(Identifier("clk".to_string()))),
                        )),
                        Box::new(Expr::Constant(2)),
                    )),
                    Box::new(Expr::Constant(1)),
                ),
                Expr::Constant(0),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multi_arithmetic_ops_different_precedence() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' + clk * 2 - 1 = 0";
    // multiplication should have higher precedence than subtraction which means it will be
    // evaluated first.
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                Expr::Subtract(
                    Box::new(Expr::Add(
                        Box::new(Expr::Next(Identifier("clk".to_string()))),
                        Box::new(Expr::Multiply(
                            Box::new(Expr::Variable(Identifier("clk".to_string()))),
                            Box::new(Expr::Constant(2)),
                        )),
                    )),
                    Box::new(Expr::Constant(1)),
                ),
                Expr::Constant(0),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}
