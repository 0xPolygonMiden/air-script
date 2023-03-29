use super::{build_parse_test, Identifier, IntegrityConstraint, Source, SourceSection};
use crate::{
    ast::{
        ConstraintType, EvaluatorFunction, EvaluatorFunctionCall, Expression::*, IntegrityStmt::*,
        Range, TraceBinding, TraceBindingAccess, TraceBindingAccessSize, Variable, VariableType,
    },
    error::{Error, ParseError},
};

// EVALUATOR FUNCTIONS
// ================================================================================================

#[test]
fn ev_fn_main_cols() {
    let source = "
    ev advance_clock(main: [clk]):
        enf clk' = clk + 1";
    let expected = Source(vec![SourceSection::EvaluatorFunction(
        EvaluatorFunction::new(
            Identifier("advance_clock".to_string()),
            vec![TraceBinding::new(Identifier("clk".to_string()), 0, 0, 1)],
            vec![Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    TraceBindingAccess(TraceBindingAccess::new(
                        Identifier("clk".to_string()),
                        0,
                        TraceBindingAccessSize::Full,
                        1,
                    )),
                    Add(
                        Box::new(Elem(Identifier("clk".to_string()))),
                        Box::new(Const(1)),
                    ),
                )),
                None,
            )],
        ),
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ev_fn_main_and_aux_cols() {
    let source = "
    ev ev_func(main: [clk], aux: [a, b]):
        let z = a + b
        enf clk' = clk + 1
        enf a' = a + z";

    let expected = Source(vec![SourceSection::EvaluatorFunction(
        EvaluatorFunction::new(
            Identifier("ev_func".to_string()),
            vec![
                TraceBinding::new(Identifier("clk".to_string()), 0, 0, 1),
                TraceBinding::new(Identifier("a".to_string()), 1, 0, 1),
                TraceBinding::new(Identifier("b".to_string()), 1, 1, 1),
            ],
            vec![
                Variable(Variable::new(
                    Identifier("z".to_string()),
                    VariableType::Scalar(Add(
                        Box::new(Elem(Identifier("a".to_string()))),
                        Box::new(Elem(Identifier("b".to_string()))),
                    )),
                )),
                Constraint(
                    ConstraintType::Inline(IntegrityConstraint::new(
                        TraceBindingAccess(TraceBindingAccess::new(
                            Identifier("clk".to_string()),
                            0,
                            TraceBindingAccessSize::Full,
                            1,
                        )),
                        Add(
                            Box::new(Elem(Identifier("clk".to_string()))),
                            Box::new(Const(1)),
                        ),
                    )),
                    None,
                ),
                Constraint(
                    ConstraintType::Inline(IntegrityConstraint::new(
                        TraceBindingAccess(TraceBindingAccess::new(
                            Identifier("a".to_string()),
                            0,
                            TraceBindingAccessSize::Full,
                            1,
                        )),
                        Add(
                            Box::new(Elem(Identifier("a".to_string()))),
                            Box::new(Elem(Identifier("z".to_string()))),
                        ),
                    )),
                    None,
                ),
            ],
        ),
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ev_fn_call_simple() {
    let source = "
    integrity_constraints:
        enf advance_clock([clk])";

    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        ConstraintType::Evaluator(EvaluatorFunctionCall::new(
            Identifier("advance_clock".to_string()),
            vec![vec![TraceBindingAccess::new(
                Identifier("clk".to_string()),
                0,
                TraceBindingAccessSize::Full,
                0,
            )]],
        )),
        None,
    )])]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ev_fn_call() {
    let source = "
    integrity_constraints:
        enf advance_clock([a, b[1], c[2..4]])";

    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        ConstraintType::Evaluator(EvaluatorFunctionCall::new(
            Identifier("advance_clock".to_string()),
            vec![vec![
                TraceBindingAccess::new(
                    Identifier("a".to_string()),
                    0,
                    TraceBindingAccessSize::Full,
                    0,
                ),
                TraceBindingAccess::new(
                    Identifier("b".to_string()),
                    1,
                    TraceBindingAccessSize::Single,
                    0,
                ),
                TraceBindingAccess::new(
                    Identifier("c".to_string()),
                    2,
                    TraceBindingAccessSize::Slice(Range::new(2, 4)),
                    0,
                ),
            ]],
        )),
        None,
    )])]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ev_fn_call_inside_ev_fn() {
    let source = "
    ev ev_func(main: [clk], aux: [a, b]):
        enf advance_clock([clk])";

    let expected = Source(vec![SourceSection::EvaluatorFunction(
        EvaluatorFunction::new(
            Identifier("ev_func".to_string()),
            vec![
                TraceBinding::new(Identifier("clk".to_string()), 0, 0, 1),
                TraceBinding::new(Identifier("a".to_string()), 1, 0, 1),
                TraceBinding::new(Identifier("b".to_string()), 1, 1, 1),
            ],
            vec![Constraint(
                ConstraintType::Evaluator(EvaluatorFunctionCall::new(
                    Identifier("advance_clock".to_string()),
                    vec![vec![TraceBindingAccess::new(
                        Identifier("clk".to_string()),
                        0,
                        TraceBindingAccessSize::Full,
                        0,
                    )]],
                )),
                None,
            )],
        ),
    )]);

    build_parse_test!(source).expect_ast(expected);
}

// INVALID USE OF EVALUATOR FUNCTIONS
// ================================================================================================

#[test]
fn ev_fn_call_with_no_args() {
    let source = "
    integrity_constraints:
        enf advance_clock()";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn ev_fn_call_with_more_than_two_args() {
    let source = "
    integrity_constraints:
        enf advance_clock([a], [b], [c])";
    let error = Error::ParseError(ParseError::InvalidEvaluatorFunction(
        "Evaluator function call must have 1 or 2 arguments".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

#[test]
fn ev_fn_with_invalid_params() {
    let source = "
    ev advance_clock():
        enf clk' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();

    let source = "
    ev advance_clock(main: [clk] aux: [a, b]):
        enf clk' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();

    let source = "
    ev advance_clock(, aux: [a, b]):
        enf clk' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();

    let source = "
    ev advance_clock(main: [clk],):
        enf clk' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();
}
