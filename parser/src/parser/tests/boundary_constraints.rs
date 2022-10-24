use super::{
    build_parse_test, Boundary, BoundaryConstraint, BoundaryConstraints, BoundaryExpr, Identifier,
    Source, SourceSection,
};

// BOUNDARY CONSTRAINTS
// ================================================================================================

#[test]
fn boundary_constraint_at_first() {
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
fn boundary_constraint_at_last() {
    let source = "
    boundary_constraints:
        enf clk.last = 15";
    let expected = Source(vec![SourceSection::BoundaryConstraints(
        BoundaryConstraints {
            boundary_constraints: vec![BoundaryConstraint::new(
                Identifier("clk".to_string()),
                Boundary::Last,
                BoundaryExpr::Const(15),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn error_invalid_boundary() {
    let source = "
    boundary_constraints:
        enf clk.0 = 15";
    build_parse_test!(source).expect_unrecognized_token();
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

#[test]
fn boundary_constraint_with_pub_input() {
    let source = "
    boundary_constraints:
        enf clk.first = a[0]";
    let expected = Source(vec![SourceSection::BoundaryConstraints(
        BoundaryConstraints {
            boundary_constraints: vec![BoundaryConstraint::new(
                Identifier("clk".to_string()),
                Boundary::First,
                BoundaryExpr::PubInput(Identifier("a".to_string()), 0),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn boundary_constraint_with_expr() {
    let source = "
    boundary_constraints:
        enf clk.first = 5 + a[3] + 6";
    let expected = Source(vec![SourceSection::BoundaryConstraints(
        BoundaryConstraints {
            boundary_constraints: vec![BoundaryConstraint::new(
                Identifier("clk".to_string()),
                Boundary::First,
                BoundaryExpr::Add(
                    Box::new(BoundaryExpr::Add(
                        Box::new(BoundaryExpr::Const(5)),
                        Box::new(BoundaryExpr::PubInput(Identifier("a".to_string()), 3)),
                    )),
                    Box::new(BoundaryExpr::Const(6)),
                ),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn err_boundary_constraint_with_identifier() {
    // TODO: ending the constraint with a gives "UnrecognizedEOF" error. These errors should be
    // improved to be more useful and consistent.
    let source = "
    boundary_constraints:
        enf clk.first = a + 5";
    build_parse_test!(source).expect_unrecognized_token();
}
