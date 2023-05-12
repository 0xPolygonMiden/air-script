use super::{Identifier, IntegrityConstraint, ParseTest, Source, SourceSection};
use crate::ast::{
    AccessType, ConstraintExpr, EvaluatorFunction, EvaluatorFunctionCall, Expression::*,
    InlineConstraintExpr, IntegrityStmt::*, Range, SymbolAccess, TraceBinding, VariableBinding,
    VariableValueExpr,
};

// EVALUATOR FUNCTIONS
// ================================================================================================

#[test]
fn ev_fn_main_cols() {
    let source = "
    ev advance_clock([clk]):
        enf clk' = clk + 1";
    let expected = Source(vec![SourceSection::EvaluatorFunction(
        EvaluatorFunction::new(
            Identifier("advance_clock".to_string()),
            vec![vec![TraceBinding::new(
                Identifier("clk".to_string()),
                0,
                0,
                1,
            )]],
            vec![Constraint(IntegrityConstraint::new(
                ConstraintExpr::Inline(InlineConstraintExpr::new(
                    SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                        1,
                    )),
                    Add(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("clk".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                        Box::new(Const(1)),
                    ),
                )),
                None,
                None,
            ))],
        ),
    )]);
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn ev_fn_aux_cols() {
    let source = "
    ev foo([], [p]):
        enf p' = p + 1";
    let expected = Source(vec![SourceSection::EvaluatorFunction(
        EvaluatorFunction::new(
            Identifier("foo".to_string()),
            vec![
                vec![],
                vec![TraceBinding::new(Identifier("p".to_string()), 1, 0, 1)],
            ],
            vec![Constraint(IntegrityConstraint::new(
                ConstraintExpr::Inline(InlineConstraintExpr::new(
                    SymbolAccess(SymbolAccess::new(
                        Identifier("p".to_string()),
                        AccessType::Default,
                        1,
                    )),
                    Add(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("p".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                        Box::new(Const(1)),
                    ),
                )),
                None,
                None,
            ))],
        ),
    )]);
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn ev_fn_main_and_aux_cols() {
    let source = "
    ev ev_func([clk], [a, b]):
        let z = a + b
        enf clk' = clk + 1
        enf a' = a + z";

    let expected = Source(vec![SourceSection::EvaluatorFunction(
        EvaluatorFunction::new(
            Identifier("ev_func".to_string()),
            vec![
                vec![TraceBinding::new(Identifier("clk".to_string()), 0, 0, 1)],
                vec![
                    TraceBinding::new(Identifier("a".to_string()), 1, 0, 1),
                    TraceBinding::new(Identifier("b".to_string()), 1, 1, 1),
                ],
            ],
            vec![
                VariableBinding(VariableBinding::new(
                    Identifier("z".to_string()),
                    VariableValueExpr::Scalar(Add(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("a".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("b".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                    )),
                )),
                Constraint(IntegrityConstraint::new(
                    ConstraintExpr::Inline(InlineConstraintExpr::new(
                        SymbolAccess(SymbolAccess::new(
                            Identifier("clk".to_string()),
                            AccessType::Default,
                            1,
                        )),
                        Add(
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("clk".to_string()),
                                AccessType::Default,
                                0,
                            ))),
                            Box::new(Const(1)),
                        ),
                    )),
                    None,
                    None,
                )),
                Constraint(IntegrityConstraint::new(
                    ConstraintExpr::Inline(InlineConstraintExpr::new(
                        SymbolAccess(SymbolAccess::new(
                            Identifier("a".to_string()),
                            AccessType::Default,
                            1,
                        )),
                        Add(
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("a".to_string()),
                                AccessType::Default,
                                0,
                            ))),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("z".to_string()),
                                AccessType::Default,
                                0,
                            ))),
                        ),
                    )),
                    None,
                    None,
                )),
            ],
        ),
    )]);
    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn ev_fn_call_simple() {
    let source = "
    integrity_constraints:
        enf advance_clock([clk])";

    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Evaluator(EvaluatorFunctionCall::new(
                Identifier("advance_clock".to_string()),
                vec![vec![SymbolAccess::new(
                    Identifier("clk".to_string()),
                    AccessType::Default,
                    0,
                )]],
            )),
            None,
            None,
        ),
    )])]);

    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn ev_fn_call() {
    let source = "
    integrity_constraints:
        enf advance_clock([a, b[1], c[2..4]])";

    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Evaluator(EvaluatorFunctionCall::new(
                Identifier("advance_clock".to_string()),
                vec![vec![
                    SymbolAccess::new(Identifier("a".to_string()), AccessType::Default, 0),
                    SymbolAccess::new(Identifier("b".to_string()), AccessType::Vector(1), 0),
                    SymbolAccess::new(
                        Identifier("c".to_string()),
                        AccessType::Slice(Range::new(2, 4)),
                        0,
                    ),
                ]],
            )),
            None,
            None,
        ),
    )])]);

    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn ev_fn_call_inside_ev_fn() {
    let source = "
    ev ev_func([clk], [a, b]):
        enf advance_clock([clk])";

    let expected = Source(vec![SourceSection::EvaluatorFunction(
        EvaluatorFunction::new(
            Identifier("ev_func".to_string()),
            vec![
                vec![TraceBinding::new(Identifier("clk".to_string()), 0, 0, 1)],
                vec![
                    TraceBinding::new(Identifier("a".to_string()), 1, 0, 1),
                    TraceBinding::new(Identifier("b".to_string()), 1, 1, 1),
                ],
            ],
            vec![Constraint(IntegrityConstraint::new(
                ConstraintExpr::Evaluator(EvaluatorFunctionCall::new(
                    Identifier("advance_clock".to_string()),
                    vec![vec![SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                        0,
                    )]],
                )),
                None,
                None,
            ))],
        ),
    )]);

    ParseTest::new().expect_ast(source, expected);
}

#[test]
fn ev_fn_call_with_more_than_two_args() {
    let source = "
    integrity_constraints:
        enf advance_clock([a], [b], [c])";

    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        IntegrityConstraint::new(
            ConstraintExpr::Evaluator(EvaluatorFunctionCall::new(
                Identifier("advance_clock".to_string()),
                vec![
                    vec![SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Default,
                        0,
                    )],
                    vec![SymbolAccess::new(
                        Identifier("b".to_string()),
                        AccessType::Default,
                        0,
                    )],
                    vec![SymbolAccess::new(
                        Identifier("c".to_string()),
                        AccessType::Default,
                        0,
                    )],
                ],
            )),
            None,
            None,
        ),
    )])]);

    ParseTest::new().expect_ast(source, expected);
}

// INVALID USE OF EVALUATOR FUNCTIONS
// ================================================================================================

#[test]
fn ev_fn_def_with_empty_final_arg() {
    let source = "
    ev ev_func([clk], []):
        enf clk' = clk + 1";
    ParseTest::new().expect_diagnostic(source, "the last trace segment cannot be empty");
}

#[test]
fn ev_fn_call_with_no_args() {
    let source = "
    integrity_constraints:
        enf advance_clock()";
    ParseTest::new().expect_unrecognized_token(source);
}

#[test]
fn ev_fn_with_invalid_params() {
    let source = "
    ev advance_clock():
        enf clk' = clk + 1";
    ParseTest::new().expect_unrecognized_token(source);

    let source = "
    ev advance_clock([clk] [a, b]):
        enf clk' = clk + 1";
    ParseTest::new().expect_unrecognized_token(source);

    let source = "
    ev advance_clock(, [a, b]):
        enf clk' = clk + 1";
    ParseTest::new().expect_unrecognized_token(source);

    let source = "
    ev advance_clock([clk],):
        enf clk' = clk + 1";
    ParseTest::new().expect_unrecognized_token(source);
}
