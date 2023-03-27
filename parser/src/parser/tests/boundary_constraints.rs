use super::{build_parse_test, Boundary, BoundaryConstraint, Identifier, Source, SourceSection};
use crate::{
    ast::{
        BoundaryStmt::*, Constant, ConstantType::*, Expression::*, MatrixAccess, PublicInput,
        TraceBindingAccess, TraceBindingAccessSize, Variable, VariableType, VectorAccess,
    },
    error::{Error, ParseError},
};

// BOUNDARY STATEMENTS
// ================================================================================================

#[test]
fn boundary_constraint_at_first() {
    let source = "
    boundary_constraints:
        enf clk.first = 0";
    let expected = Source(vec![SourceSection::BoundaryConstraints(vec![Constraint(
        BoundaryConstraint::new(
            TraceBindingAccess::new(
                Identifier("clk".to_string()),
                0,
                TraceBindingAccessSize::Full,
                0,
            ),
            Boundary::First,
            Const(0),
        ),
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn boundary_constraint_at_last() {
    let source = "
    boundary_constraints:
        enf clk.last = 15";
    let expected = Source(vec![SourceSection::BoundaryConstraints(vec![Constraint(
        BoundaryConstraint::new(
            TraceBindingAccess::new(
                Identifier("clk".to_string()),
                0,
                TraceBindingAccessSize::Full,
                0,
            ),
            Boundary::Last,
            Const(15),
        ),
    )])]);
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
        Constraint(BoundaryConstraint::new(
            TraceBindingAccess::new(
                Identifier("clk".to_string()),
                0,
                TraceBindingAccessSize::Full,
                0,
            ),
            Boundary::First,
            Const(0),
        )),
        Constraint(BoundaryConstraint::new(
            TraceBindingAccess::new(
                Identifier("clk".to_string()),
                0,
                TraceBindingAccessSize::Full,
                0,
            ),
            Boundary::Last,
            Const(1),
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
        SourceSection::BoundaryConstraints(vec![Constraint(BoundaryConstraint::new(
            TraceBindingAccess::new(
                Identifier("clk".to_string()),
                0,
                TraceBindingAccessSize::Full,
                0,
            ),
            Boundary::First,
            VectorAccess(VectorAccess::new(Identifier("a".to_string()), 0)),
        ))]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn boundary_constraint_with_expr() {
    let source = "
    boundary_constraints:
        enf clk.first = 5 + a[3] + 6";
    let expected = Source(vec![SourceSection::BoundaryConstraints(vec![Constraint(
        BoundaryConstraint::new(
            TraceBindingAccess::new(
                Identifier("clk".to_string()),
                0,
                TraceBindingAccessSize::Full,
                0,
            ),
            Boundary::First,
            Add(
                Box::new(Add(
                    Box::new(Const(5)),
                    Box::new(VectorAccess(VectorAccess::new(
                        Identifier("a".to_string()),
                        3,
                    ))),
                )),
                Box::new(Const(6)),
            ),
        ),
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn boundary_constraint_with_const() {
    let source = "
    const A = 1
    const B = [0, 1]
    const C = [[0, 1], [1, 0]]
    boundary_constraints:
        enf clk.first = A + B[1] - C[0][1]";
    let expected = Source(vec![
        SourceSection::Constant(Constant::new(Identifier("A".to_string()), Scalar(1))),
        SourceSection::Constant(Constant::new(
            Identifier("B".to_string()),
            Vector(vec![0, 1]),
        )),
        SourceSection::Constant(Constant::new(
            Identifier("C".to_string()),
            Matrix(vec![vec![0, 1], vec![1, 0]]),
        )),
        SourceSection::BoundaryConstraints(vec![Constraint(BoundaryConstraint::new(
            TraceBindingAccess::new(
                Identifier("clk".to_string()),
                0,
                TraceBindingAccessSize::Full,
                0,
            ),
            Boundary::First,
            Sub(
                Box::new(Add(
                    Box::new(Elem(Identifier("A".to_string()))),
                    Box::new(VectorAccess(VectorAccess::new(
                        Identifier("B".to_string()),
                        1,
                    ))),
                )),
                Box::new(MatrixAccess(MatrixAccess::new(
                    Identifier("C".to_string()),
                    0,
                    1,
                ))),
            ),
        ))]),
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
        Variable(Variable::new(
            Identifier("a".to_string()),
            VariableType::Scalar(Exp(Box::new(Const(2)), Box::new(Const(2)))),
        )),
        Variable(Variable::new(
            Identifier("b".to_string()),
            VariableType::Vector(vec![
                Elem(Identifier("a".to_string())),
                Mul(
                    Box::new(Const(2)),
                    Box::new(Elem(Identifier("a".to_string()))),
                ),
            ]),
        )),
        Variable(Variable::new(
            Identifier("c".to_string()),
            VariableType::Matrix(vec![
                vec![
                    Sub(
                        Box::new(Elem(Identifier("a".to_string()))),
                        Box::new(Const(1)),
                    ),
                    Exp(
                        Box::new(Elem(Identifier("a".to_string()))),
                        Box::new(Const(2)),
                    ),
                ],
                vec![
                    VectorAccess(VectorAccess::new(Identifier("b".to_string()), 0)),
                    VectorAccess(VectorAccess::new(Identifier("b".to_string()), 1)),
                ],
            ]),
        )),
        Constraint(BoundaryConstraint::new(
            TraceBindingAccess::new(
                Identifier("clk".to_string()),
                0,
                TraceBindingAccessSize::Full,
                0,
            ),
            Boundary::First,
            Add(
                Box::new(Add(
                    Box::new(Const(5)),
                    Box::new(VectorAccess(VectorAccess::new(
                        Identifier("a".to_string()),
                        3,
                    ))),
                )),
                Box::new(Const(6)),
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
    integrity_constraints:
        enf clk' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();
}
