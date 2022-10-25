use super::{
    build_parse_test, Boundary, BoundaryConstraint, BoundaryConstraints, BoundaryExpr, Identifier,
    Source, SourceSection,
};

// SECTIONS
// ================================================================================================

#[test]
fn boundary_constraints() {
    let source = "
    boundary_constraints:
        enf clk.first = 0";
    let expected = Source(vec![SourceSection::BoundaryConstraints(
        BoundaryConstraints {
            boundary_constraints: vec![BoundaryConstraint::new(
                Identifier("clk".to_string()),
                Boundary::First,
                BoundaryExpr::Const(0),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multiple_boundary_constraints() {
    let source = "
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1";
    let expected = Source(vec![SourceSection::BoundaryConstraints(
        BoundaryConstraints {
            boundary_constraints: vec![
                BoundaryConstraint::new(
                    Identifier("clk".to_string()),
                    Boundary::First,
                    BoundaryExpr::Const(0),
                ),
                BoundaryConstraint::new(
                    Identifier("clk".to_string()),
                    Boundary::Last,
                    BoundaryExpr::Const(1),
                ),
            ],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}
