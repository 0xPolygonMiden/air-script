use super::{
    build_parse_test, Boundary, BoundaryConstraint, BoundaryConstraints, BoundaryExpr, Identifier,
    Source, SourceSection,
};
use crate::ast::constants::{
    Constant,
    ConstantType::{Matrix, Scalar, Vector},
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
                BoundaryExpr::VecElem(Identifier("a".to_string()), 0),
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
                        Box::new(BoundaryExpr::VecElem(Identifier("a".to_string()), 3)),
                    )),
                    Box::new(BoundaryExpr::Const(6)),
                ),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn boundary_constraint_with_const() {
    let source = "
    constants:
        a: 1
        b: [0, 1]
        c: [[0, 1], [1, 0]]
    boundary_constraints:
        enf clk.first = a + b[1] - c[0][1]";
    let expected = Source(vec![
        SourceSection::Constants(vec![
            Constant {
                name: Identifier("a".to_string()),
                value: Scalar(1),
            },
            Constant {
                name: Identifier("b".to_string()),
                value: Vector(vec![0, 1]),
            },
            Constant {
                name: Identifier("c".to_string()),
                value: Matrix(vec![vec![0, 1], vec![1, 0]]),
            },
        ]),
        SourceSection::BoundaryConstraints(BoundaryConstraints {
            boundary_constraints: vec![BoundaryConstraint::new(
                Identifier("clk".to_string()),
                Boundary::First,
                BoundaryExpr::Sub(
                    Box::new(BoundaryExpr::Add(
                        Box::new(BoundaryExpr::Var(Identifier("a".to_string()))),
                        Box::new(BoundaryExpr::VecElem(Identifier("b".to_string()), 1)),
                    )),
                    Box::new(BoundaryExpr::MatrixElem(Identifier("c".to_string()), 0, 1)),
                ),
            )],
        }),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn err_empty_boundary_constraints() {
    let source = "
    boundary_constraints:
    transition_constraints:
        enf clk' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();
}
