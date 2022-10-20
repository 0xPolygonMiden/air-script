use super::{
    build_parse_test, Expr, Identifier, Source, SourceSection, TransitionConstraint,
    TransitionConstraints,
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
