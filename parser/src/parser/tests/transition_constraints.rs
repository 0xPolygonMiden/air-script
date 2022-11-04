use super::{
    build_parse_test, Identifier, Source, SourceSection, TransitionConstraint, TransitionExpr,
};

// TRANSITION CONSTRAINTS
// ================================================================================================

#[test]
fn transition_constraints() {
    let source = "
    transition_constraints:
        enf clk' = clk + 1";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Next(Identifier("clk".to_string())),
            TransitionExpr::Add(
                Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
                Box::new(TransitionExpr::Const(1)),
            ),
        ),
    ])]);
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
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Next(Identifier("clk".to_string())),
            TransitionExpr::Add(
                Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
                Box::new(TransitionExpr::Const(1)),
            ),
        ),
        TransitionConstraint::new(
            TransitionExpr::Sub(
                Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                Box::new(TransitionExpr::Var(Identifier("clk".to_string()))),
            ),
            TransitionExpr::Const(1),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraint_with_periodic_col() {
    let source = "
    transition_constraints:
        enf k0 + b = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Add(
                Box::new(TransitionExpr::Var(Identifier("k0".to_string()))),
                Box::new(TransitionExpr::Var(Identifier("b".to_string()))),
            ),
            TransitionExpr::Const(0),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraint_with_random_value() {
    let source = "
    transition_constraints:
        enf a + $rand[1] = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(vec![
        TransitionConstraint::new(
            TransitionExpr::Add(
                Box::new(TransitionExpr::Var(Identifier("a".to_string()))),
                Box::new(TransitionExpr::Rand(1)),
            ),
            TransitionExpr::Const(0),
        ),
    ])]);
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
