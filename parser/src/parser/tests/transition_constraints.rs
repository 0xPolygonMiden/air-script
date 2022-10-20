use super::{
    build_parse_test, Expr, Identifier, Source, SourceSection, TransitionConstraint,
    TransitionConstraints,
};

// SECTIONS
// ================================================================================================

#[test]
fn transition_constraints() {
    let source = "
    transition_constraints:
        enf clk' = clk + 1";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                Expr::Next(Identifier("clk".to_string())),
                Expr::Add(
                    Box::new(Expr::Variable(Identifier("clk".to_string()))),
                    Box::new(Expr::Constant(1)),
                ),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraints_invalid() {
    let source = "transition_constraints:
        enf clk' = clk = 1";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn multiple_transition_constraints() {
    let source = "
    transition_constraints:
        enf clk' = clk + 1
        enf clk' - clk = 1";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![
                TransitionConstraint::new(
                    Expr::Next(Identifier("clk".to_string())),
                    Expr::Add(
                        Box::new(Expr::Variable(Identifier("clk".to_string()))),
                        Box::new(Expr::Constant(1)),
                    ),
                ),
                TransitionConstraint::new(
                    Expr::Subtract(
                        Box::new(Expr::Next(Identifier("clk".to_string()))),
                        Box::new(Expr::Variable(Identifier("clk".to_string()))),
                    ),
                    Expr::Constant(1),
                ),
            ],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

// UNRECOGNIZED TOKEN ERRORS
// ================================================================================================

#[test]
fn error_invalid_next_usage() {
    let source = "
    transition_constraints:
        enf clk'' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();
}
