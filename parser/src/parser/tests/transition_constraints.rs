use super::{
    build_parse_test, Identifier, Source, SourceSection::*, TransitionConstraint, TransitionExpr::*,
};
use crate::{
    ast::{
        constants::{Constant, ConstantType::Matrix, ConstantType::Scalar, ConstantType::Vector},
        MatrixAccess,
        TransitionStmt::*,
        TransitionVariable, TransitionVariableType, VectorAccess,
    },
    error::{Error, ParseError},
};

// TRANSITION CONSTRAINTS
// ================================================================================================

#[test]
fn transition_constraints() {
    let source = "
    transition_constraints:
        enf clk' = clk + 1";
    let expected = Source(vec![TransitionConstraints(vec![Constraint(
        TransitionConstraint::new(
            Next(Identifier("clk".to_string())),
            Add(
                Box::new(Elem(Identifier("clk".to_string()))),
                Box::new(Const(1)),
            ),
        ),
    )])]);
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
    let expected = Source(vec![TransitionConstraints(vec![
        Constraint(TransitionConstraint::new(
            Next(Identifier("clk".to_string())),
            Add(
                Box::new(Elem(Identifier("clk".to_string()))),
                Box::new(Const(1)),
            ),
        )),
        Constraint(TransitionConstraint::new(
            Sub(
                Box::new(Next(Identifier("clk".to_string()))),
                Box::new(Elem(Identifier("clk".to_string()))),
            ),
            Const(1),
        )),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraint_with_periodic_col() {
    let source = "
    transition_constraints:
        enf k0 + b = 0";
    let expected = Source(vec![TransitionConstraints(vec![Constraint(
        TransitionConstraint::new(
            Add(
                Box::new(Elem(Identifier("k0".to_string()))),
                Box::new(Elem(Identifier("b".to_string()))),
            ),
            Const(0),
        ),
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraint_with_random_value() {
    let source = "
    transition_constraints:
        enf a + $rand[1] = 0";
    let expected = Source(vec![TransitionConstraints(vec![Constraint(
        TransitionConstraint::new(
            Add(
                Box::new(Elem(Identifier("a".to_string()))),
                Box::new(Rand(1)),
            ),
            Const(0),
        ),
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraint_with_constants() {
    let source = "
    constants:
        A: 0
        B: [0, 1]
        C: [[0, 1], [1, 0]]
    transition_constraints:
        enf clk + A = B[1] + C[1][1]";
    let expected = Source(vec![
        Constants(vec![
            Constant::new(Identifier("A".to_string()), Scalar(0)),
            Constant::new(Identifier("B".to_string()), Vector(vec![0, 1])),
            Constant::new(
                Identifier("C".to_string()),
                Matrix(vec![vec![0, 1], vec![1, 0]]),
            ),
        ]),
        TransitionConstraints(vec![Constraint(TransitionConstraint::new(
            Add(
                Box::new(Elem(Identifier("clk".to_string()))),
                Box::new(Elem(Identifier("A".to_string()))),
            ),
            Add(
                Box::new(VectorAccess(VectorAccess::new(
                    Identifier("B".to_string()),
                    1,
                ))),
                Box::new(MatrixAccess(MatrixAccess::new(
                    Identifier("C".to_string()),
                    1,
                    1,
                ))),
            ),
        ))]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraint_with_variables() {
    let source = "
    transition_constraints:
        let a = 2^2
        let b = [a, 2 * a]
        let c = [[a - 1, a^2], [b[0], b[1]]]
        enf clk + a = b[1] + c[1][1]";
    let expected = Source(vec![TransitionConstraints(vec![
        Variable(TransitionVariable::new(
            Identifier("a".to_string()),
            TransitionVariableType::Scalar(Exp(Box::new(Const(2)), 2)),
        )),
        Variable(TransitionVariable::new(
            Identifier("b".to_string()),
            TransitionVariableType::Vector(vec![
                Elem(Identifier("a".to_string())),
                Mul(
                    Box::new(Const(2)),
                    Box::new(Elem(Identifier("a".to_string()))),
                ),
            ]),
        )),
        Variable(TransitionVariable::new(
            Identifier("c".to_string()),
            TransitionVariableType::Matrix(vec![
                vec![
                    Sub(
                        Box::new(Elem(Identifier("a".to_string()))),
                        Box::new(Const(1)),
                    ),
                    Exp(Box::new(Elem(Identifier("a".to_string()))), 2),
                ],
                vec![
                    VectorAccess(VectorAccess::new(Identifier("b".to_string()), 0)),
                    VectorAccess(VectorAccess::new(Identifier("b".to_string()), 1)),
                ],
            ]),
        )),
        Constraint(TransitionConstraint::new(
            Add(
                Box::new(Elem(Identifier("clk".to_string()))),
                Box::new(Elem(Identifier("a".to_string()))),
            ),
            Add(
                Box::new(VectorAccess(VectorAccess::new(
                    Identifier("b".to_string()),
                    1,
                ))),
                Box::new(MatrixAccess(MatrixAccess::new(
                    Identifier("c".to_string()),
                    1,
                    1,
                ))),
            ),
        )),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn err_missing_transition_constraint() {
    let source = "
    transition_constraints:
        let a = 2^2
        let b = [a, 2 * a]
        let c = [[a - 1, a^2], [b[0], b[1]]]";
    let error = Error::ParseError(ParseError::MissingTransitionConstraint(
        "Declaration of at least one transition constraint is required".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
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

#[test]
fn err_empty_transition_constraints() {
    let source = "
    transition_constraints:
        
    boundary_constraints:
        enf clk.first = 1";
    build_parse_test!(source).expect_unrecognized_token();
}
