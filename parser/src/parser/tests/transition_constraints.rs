use super::{
    build_parse_test, Identifier, Source, SourceSection, TransitionConstraint,
    TransitionConstraints, TransitionExpr,
};

// TRANSITION CONSTRAINTS
// ================================================================================================

#[test]
fn transition_constraints() {
    let source = "
    transition_constraints:
        enf clk' = clk + 1";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                TransitionExpr::Next(Identifier("clk".to_string())),
                TransitionExpr::Add(
                    Box::new(TransitionExpr::Variable(Identifier("clk".to_string()))),
                    Box::new(TransitionExpr::Constant(1)),
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
                    TransitionExpr::Next(Identifier("clk".to_string())),
                    TransitionExpr::Add(
                        Box::new(TransitionExpr::Variable(Identifier("clk".to_string()))),
                        Box::new(TransitionExpr::Constant(1)),
                    ),
                ),
                TransitionConstraint::new(
                    TransitionExpr::Subtract(
                        Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                        Box::new(TransitionExpr::Variable(Identifier("clk".to_string()))),
                    ),
                    TransitionExpr::Constant(1),
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
