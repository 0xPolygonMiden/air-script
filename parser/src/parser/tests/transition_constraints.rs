use super::{
    build_parse_test, Identifier, Source, SourceSection, TransitionConstraint,
    TransitionConstraints, TransitionExpr,
};
use crate::ast::{
    constants::{Constant, ConstantType::Matrix, ConstantType::Scalar, ConstantType::Vector},
    MatrixAccess, VectorAccess,
};

// TRANSITION CONSTRAINTS
// ================================================================================================

#[test]
fn transition_constraints() {
    let source = "
    transition_constraints:
        enf clk' = clk + 1";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                TransitionExpr::Next(Identifier("clk".to_string())),
                TransitionExpr::Add(
                    Box::new(TransitionExpr::Elem(Identifier("clk".to_string()))),
                    Box::new(TransitionExpr::Const(1)),
                ),
            )],
        },
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
        TransitionConstraints {
            transition_constraints: vec![
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
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraint_with_periodic_col() {
    let source = "
    transition_constraints:
        enf k0 + b = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                TransitionExpr::Add(
                    Box::new(TransitionExpr::Elem(Identifier("k0".to_string()))),
                    Box::new(TransitionExpr::Elem(Identifier("b".to_string()))),
                ),
                TransitionExpr::Const(0),
            )],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraint_with_random_value() {
    let source = "
    transition_constraints:
        enf a + $rand[1] = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                TransitionExpr::Add(
                    Box::new(TransitionExpr::Elem(Identifier("a".to_string()))),
                    Box::new(TransitionExpr::Rand(1)),
                ),
                TransitionExpr::Const(0),
            )],
        },
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
        SourceSection::TransitionConstraints(TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
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
        }),
    ]);
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

#[test]
fn err_empty_transition_constraints() {
    let source = "
    transition_constraints:
        
    boundary_constraints:
        enf clk.first = 1";
    build_parse_test!(source).expect_unrecognized_token();
}
