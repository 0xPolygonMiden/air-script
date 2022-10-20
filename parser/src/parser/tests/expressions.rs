use super::{
    build_parse_test, Identifier, Source, SourceSection, TransitionConstraint,
    TransitionConstraints, TransitionExpr,
};

// SECTIONS
// ================================================================================================

#[test]
fn multi_arithmetic_ops() {
    // the operation must be put into a source section, or parsing will fail
    let source = "
    transition_constraints:
        enf clk' - clk - 1 = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                TransitionExpr::Subtract(
                    Box::new(TransitionExpr::Subtract(
                        Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                        Box::new(TransitionExpr::Variable(Identifier("clk".to_string()))),
                    )),
                    Box::new(TransitionExpr::Constant(1)),
                ),
                TransitionExpr::Constant(0),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}
