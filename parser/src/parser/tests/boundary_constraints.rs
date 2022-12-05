use super::{
    build_parse_test, Boundary, BoundaryConstraint, BoundaryExpr, Identifier, Source, SourceSection,
};
use crate::{
    ast::{
        constants::{
            Constant,
            ConstantType::{Matrix, Scalar, Vector},
        },
        BoundaryStmt, BoundaryVariable, BoundaryVariableType, MatrixAccess, PublicInput,
        VectorAccess,
    },
    error::{Error, ParseError},
};

// BOUNDARY CONSTRAINTS
// ================================================================================================

#[test]
fn boundary_constraint_at_first() {
    let source = "
    boundary_constraints:
        enf clk.first = 0";
    let expected = Source(vec![SourceSection::BoundaryConstraints(vec![
        BoundaryStmt::Constraint(BoundaryConstraint::new(
            Identifier("clk".to_string()),
            Boundary::First,
            BoundaryExpr::Const(0),
        )),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn boundary_constraint_at_last() {
    let source = "
    boundary_constraints:
        enf clk.last = 15";
    let expected = Source(vec![SourceSection::BoundaryConstraints(vec![
        BoundaryStmt::Constraint(BoundaryConstraint::new(
            Identifier("clk".to_string()),
            Boundary::Last,
            BoundaryExpr::Const(15),
        )),
    ])]);
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
    let expected = Source(vec![SourceSection::BoundaryConstraints(vec![
        BoundaryStmt::Constraint(BoundaryConstraint::new(
            Identifier("clk".to_string()),
            Boundary::First,
            BoundaryExpr::Const(0),
        )),
        BoundaryStmt::Constraint(BoundaryConstraint::new(
            Identifier("clk".to_string()),
            Boundary::Last,
            BoundaryExpr::Const(1),
        )),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn boundary_constraint_with_pub_input() {
    let source = "
    public_inputs:
        a: [16]
    boundary_constraints:
        enf clk.first = a[0]";
    let expected = Source(vec![
        SourceSection::PublicInputs(vec![PublicInput::new(Identifier("a".to_string()), 16)]),
        SourceSection::BoundaryConstraints(vec![BoundaryStmt::Constraint(
            BoundaryConstraint::new(
                Identifier("clk".to_string()),
                Boundary::First,
                BoundaryExpr::VectorAccess(VectorAccess::new(Identifier("a".to_string()), 0)),
            ),
        )]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn boundary_constraint_with_expr() {
    let source = "
    boundary_constraints:
        enf clk.first = 5 + a[3] + 6";
    let expected = Source(vec![SourceSection::BoundaryConstraints(vec![
        BoundaryStmt::Constraint(BoundaryConstraint::new(
            Identifier("clk".to_string()),
            Boundary::First,
            BoundaryExpr::Add(
                Box::new(BoundaryExpr::Add(
                    Box::new(BoundaryExpr::Const(5)),
                    Box::new(BoundaryExpr::VectorAccess(VectorAccess::new(
                        Identifier("a".to_string()),
                        3,
                    ))),
                )),
                Box::new(BoundaryExpr::Const(6)),
            ),
        )),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn boundary_constraint_with_const() {
    let source = "
    constants:
        A: 1
        B: [0, 1]
        C: [[0, 1], [1, 0]]
    boundary_constraints:
        enf clk.first = A + B[1] - C[0][1]";
    let expected = Source(vec![
        SourceSection::Constants(vec![
            Constant::new(Identifier("A".to_string()), Scalar(1)),
            Constant::new(Identifier("B".to_string()), Vector(vec![0, 1])),
            Constant::new(
                Identifier("C".to_string()),
                Matrix(vec![vec![0, 1], vec![1, 0]]),
            ),
        ]),
        SourceSection::BoundaryConstraints(vec![BoundaryStmt::Constraint(
            BoundaryConstraint::new(
                Identifier("clk".to_string()),
                Boundary::First,
                BoundaryExpr::Sub(
                    Box::new(BoundaryExpr::Add(
                        Box::new(BoundaryExpr::Elem(Identifier("A".to_string()))),
                        Box::new(BoundaryExpr::VectorAccess(VectorAccess::new(
                            Identifier("B".to_string()),
                            1,
                        ))),
                    )),
                    Box::new(BoundaryExpr::MatrixAccess(MatrixAccess::new(
                        Identifier("C".to_string()),
                        0,
                        1,
                    ))),
                ),
            ),
        )]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn boundary_constraint_with_variables() {
    let source = "
    boundary_constraints:
        let a = 2^2
        let b = [a, 2 * a]
        let c = [[a - 1, a^2], [b[0], b[1]]]
        enf clk.first = 5 + a[3] + 6";
    let expected = Source(vec![SourceSection::BoundaryConstraints(vec![
        BoundaryStmt::Variable(BoundaryVariable::new(
            Identifier("a".to_string()),
            BoundaryVariableType::Scalar(BoundaryExpr::Exp(Box::new(BoundaryExpr::Const(2)), 2)),
        )),
        BoundaryStmt::Variable(BoundaryVariable::new(
            Identifier("b".to_string()),
            BoundaryVariableType::Vector(vec![
                BoundaryExpr::Elem(Identifier("a".to_string())),
                BoundaryExpr::Mul(
                    Box::new(BoundaryExpr::Const(2)),
                    Box::new(BoundaryExpr::Elem(Identifier("a".to_string()))),
                ),
            ]),
        )),
        BoundaryStmt::Variable(BoundaryVariable::new(
            Identifier("c".to_string()),
            BoundaryVariableType::Matrix(vec![
                vec![
                    BoundaryExpr::Sub(
                        Box::new(BoundaryExpr::Elem(Identifier("a".to_string()))),
                        Box::new(BoundaryExpr::Const(1)),
                    ),
                    BoundaryExpr::Exp(Box::new(BoundaryExpr::Elem(Identifier("a".to_string()))), 2),
                ],
                vec![
                    BoundaryExpr::VectorAccess(VectorAccess::new(Identifier("b".to_string()), 0)),
                    BoundaryExpr::VectorAccess(VectorAccess::new(Identifier("b".to_string()), 1)),
                ],
            ]),
        )),
        BoundaryStmt::Constraint(BoundaryConstraint::new(
            Identifier("clk".to_string()),
            Boundary::First,
            BoundaryExpr::Add(
                Box::new(BoundaryExpr::Add(
                    Box::new(BoundaryExpr::Const(5)),
                    Box::new(BoundaryExpr::VectorAccess(VectorAccess::new(
                        Identifier("a".to_string()),
                        3,
                    ))),
                )),
                Box::new(BoundaryExpr::Const(6)),
            ),
        )),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn err_invalid_variable() {
    let source = "
    boundary_constraints:
        let a = 2^2 + [1]";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn err_missing_boundary_constraint() {
    let source = "
    boundary_constraints:
        let a = 2^2
        let b = [a, 2 * a]
        let c = [[a - 1, a^2], [b[0], b[1]]]";
    let error = Error::ParseError(ParseError::MissingBoundaryConstraint(
        "Declaration of at least one boundary constraint is required".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

#[test]
fn err_empty_boundary_constraints() {
    let source = "
    boundary_constraints:
    transition_constraints:
        enf clk' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();
}
