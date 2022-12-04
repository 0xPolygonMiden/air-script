use super::{
    build_parse_test, Identifier, Source, SourceSection, TransitionConstraint,
    TransitionConstraints, TransitionExpr,
};
use crate::{
    ast::{
        constants::{Constant, ConstantType::Matrix, ConstantType::Scalar, ConstantType::Vector},
        MatrixAccess, TransitionVariable, TransitionVariableType, VectorAccess,
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
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints::new(
            vec![],
            vec![TransitionConstraint::new(
                TransitionExpr::Next(Identifier("clk".to_string())),
                TransitionExpr::Add(
                    Box::new(TransitionExpr::Elem(Identifier("clk".to_string()))),
                    Box::new(TransitionExpr::Const(1)),
                ),
            )],
        ),
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
        TransitionConstraints::new(
            vec![],
            vec![
                TransitionConstraint::new(
                    TransitionExpr::Next(Identifier("clk".to_string())),
                    TransitionExpr::Add(
                        Box::new(TransitionExpr::Elem(Identifier("clk".to_string()))),
                        Box::new(TransitionExpr::Const(1)),
                    ),
                ),
                TransitionConstraint::new(
                    TransitionExpr::Sub(
                        Box::new(TransitionExpr::Next(Identifier("clk".to_string()))),
                        Box::new(TransitionExpr::Elem(Identifier("clk".to_string()))),
                    ),
                    TransitionExpr::Const(1),
                ),
            ],
        ),
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraint_with_periodic_col() {
    let source = "
    transition_constraints:
        enf k0 + b = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints::new(
            vec![],
            vec![TransitionConstraint::new(
                TransitionExpr::Add(
                    Box::new(TransitionExpr::Elem(Identifier("k0".to_string()))),
                    Box::new(TransitionExpr::Elem(Identifier("b".to_string()))),
                ),
                TransitionExpr::Const(0),
            )],
        ),
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraint_with_random_value() {
    let source = "
    transition_constraints:
        enf a + $rand[1] = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints::new(
            vec![],
            vec![TransitionConstraint::new(
                TransitionExpr::Add(
                    Box::new(TransitionExpr::Elem(Identifier("a".to_string()))),
                    Box::new(TransitionExpr::Rand(1)),
                ),
                TransitionExpr::Const(0),
            )],
        ),
    )]);
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
        SourceSection::Constants(vec![
            Constant::new(Identifier("A".to_string()), Scalar(0)),
            Constant::new(Identifier("B".to_string()), Vector(vec![0, 1])),
            Constant::new(
                Identifier("C".to_string()),
                Matrix(vec![vec![0, 1], vec![1, 0]]),
            ),
        ]),
        SourceSection::TransitionConstraints(TransitionConstraints::new(
            vec![],
            vec![TransitionConstraint::new(
                TransitionExpr::Add(
                    Box::new(TransitionExpr::Elem(Identifier("clk".to_string()))),
                    Box::new(TransitionExpr::Elem(Identifier("A".to_string()))),
                ),
                TransitionExpr::Add(
                    Box::new(TransitionExpr::VectorAccess(VectorAccess::new(
                        Identifier("B".to_string()),
                        1,
                    ))),
                    Box::new(TransitionExpr::MatrixAccess(MatrixAccess::new(
                        Identifier("C".to_string()),
                        1,
                        1,
                    ))),
                ),
            )],
        )),
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
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints::new(
            vec![
                TransitionVariable::new(
                    Identifier("a".to_string()),
                    TransitionVariableType::Scalar(TransitionExpr::Exp(
                        Box::new(TransitionExpr::Const(2)),
                        2,
                    )),
                ),
                TransitionVariable::new(
                    Identifier("b".to_string()),
                    TransitionVariableType::Vector(vec![
                        TransitionExpr::Elem(Identifier("a".to_string())),
                        TransitionExpr::Mul(
                            Box::new(TransitionExpr::Const(2)),
                            Box::new(TransitionExpr::Elem(Identifier("a".to_string()))),
                        ),
                    ]),
                ),
                TransitionVariable::new(
                    Identifier("c".to_string()),
                    TransitionVariableType::Matrix(vec![
                        vec![
                            TransitionExpr::Sub(
                                Box::new(TransitionExpr::Elem(Identifier("a".to_string()))),
                                Box::new(TransitionExpr::Const(1)),
                            ),
                            TransitionExpr::Exp(
                                Box::new(TransitionExpr::Elem(Identifier("a".to_string()))),
                                2,
                            ),
                        ],
                        vec![
                            TransitionExpr::VectorAccess(VectorAccess::new(
                                Identifier("b".to_string()),
                                0,
                            )),
                            TransitionExpr::VectorAccess(VectorAccess::new(
                                Identifier("b".to_string()),
                                1,
                            )),
                        ],
                    ]),
                ),
            ],
            vec![TransitionConstraint::new(
                TransitionExpr::Add(
                    Box::new(TransitionExpr::Elem(Identifier("clk".to_string()))),
                    Box::new(TransitionExpr::Elem(Identifier("a".to_string()))),
                ),
                TransitionExpr::Add(
                    Box::new(TransitionExpr::VectorAccess(VectorAccess::new(
                        Identifier("b".to_string()),
                        1,
                    ))),
                    Box::new(TransitionExpr::MatrixAccess(MatrixAccess::new(
                        Identifier("c".to_string()),
                        1,
                        1,
                    ))),
                ),
            )],
        ),
    )]);
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
