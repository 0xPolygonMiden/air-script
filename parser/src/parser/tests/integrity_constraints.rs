use super::{build_parse_test, Identifier, IntegrityConstraint, Source, SourceSection};
use crate::{
    ast::{
        Constant, ConstantType::*, ConstraintType, Expression::*, IndexedTraceAccess,
        IntegrityStmt::*, MatrixAccess, TraceBindingAccess, TraceBindingAccessSize, Variable,
        VariableType, VectorAccess,
    },
    error::{Error, ParseError},
};

// INTEGRITY STATEMENTS
// ================================================================================================

#[test]
fn integrity_constraints() {
    let source = "
    integrity_constraints:
        enf clk' = clk + 1";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
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
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multiple_integrity_constraints() {
    let source = "
    integrity_constraints:
        enf clk' = clk + 1
        enf clk' - clk = 1";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![
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
                Sub(
                    Box::new(TraceBindingAccess(TraceBindingAccess::new(
                        Identifier("clk".to_string()),
                        0,
                        TraceBindingAccessSize::Full,
                        1,
                    ))),
                    Box::new(Elem(Identifier("clk".to_string()))),
                ),
                Const(1),
            )),
            None,
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_with_periodic_col() {
    let source = "
    integrity_constraints:
        enf k0 + b = 0";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        ConstraintType::Inline(IntegrityConstraint::new(
            Add(
                Box::new(Elem(Identifier("k0".to_string()))),
                Box::new(Elem(Identifier("b".to_string()))),
            ),
            Const(0),
        )),
        None,
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_with_random_value() {
    let source = "
    integrity_constraints:
        enf a + $rand[1] = 0";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        ConstraintType::Inline(IntegrityConstraint::new(
            Add(
                Box::new(Elem(Identifier("a".to_string()))),
                Box::new(Rand(Identifier("rand".to_string()), 1)),
            ),
            Const(0),
        )),
        None,
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_with_constants() {
    let source = "
        const A = 0
        const B = [0, 1]
        const C = [[0, 1], [1, 0]]
    integrity_constraints:
        enf clk + A = B[1] + C[1][1]";
    let expected = Source(vec![
        SourceSection::Constant(Constant::new(Identifier("A".to_string()), Scalar(0))),
        SourceSection::Constant(Constant::new(
            Identifier("B".to_string()),
            Vector(vec![0, 1]),
        )),
        SourceSection::Constant(Constant::new(
            Identifier("C".to_string()),
            Matrix(vec![vec![0, 1], vec![1, 0]]),
        )),
        SourceSection::IntegrityConstraints(vec![Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
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
            )),
            None,
        )]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_with_variables() {
    let source = "
    integrity_constraints:
        let a = 2^2
        let b = [a, 2 * a]
        let c = [[a - 1, a^2], [b[0], b[1]]]
        enf clk + a = b[1] + c[1][1]";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![
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
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
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
            None,
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_with_indexed_trace_access() {
    let source = "
    integrity_constraints:
        enf $main[0]' = $main[1] + 1
        enf $aux[0]' - $aux[1] = 1";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                TraceAccess(IndexedTraceAccess::new(0, 0, 1, 1)),
                Add(
                    Box::new(TraceAccess(IndexedTraceAccess::new(0, 1, 1, 0))),
                    Box::new(Const(1)),
                ),
            )),
            None,
        ),
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                Sub(
                    Box::new(TraceAccess(IndexedTraceAccess::new(1, 0, 1, 1))),
                    Box::new(TraceAccess(IndexedTraceAccess::new(1, 1, 1, 0))),
                ),
                Const(1),
            )),
            None,
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

// CONSTRAINT COMPREHENSION
// ================================================================================================

#[test]
fn ic_comprehension_one_iterable_identifier() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    integrity_constraints:
        enf x = a + b for x in c";

    let expected = Source(vec![
        Trace(Trace {
            main_cols: vec![
                TraceCols::new(Identifier("a".to_string()), 1),
                TraceCols::new(Identifier("b".to_string()), 1),
                TraceCols::new(Identifier("c".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        IntegrityConstraints(vec![ConstraintComprehension(
            ConstraintType::Inline(IntegrityConstraint::new(
                Elem(Identifier("x".to_string())),
                Add(
                    Box::new(Elem(Identifier("a".to_string()))),
                    Box::new(Elem(Identifier("b".to_string()))),
                ),
            )),
            None,
            vec![(
                Identifier("x".to_string()),
                Iterable::Identifier(Identifier("c".to_string())),
            )],
        )]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_comprehension_one_iterable_range() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]
    
    integrity_constraints:
        enf x = a + b for x in (1..4)";

    let expected = Source(vec![
        Trace(Trace {
            main_cols: vec![
                TraceCols::new(Identifier("a".to_string()), 1),
                TraceCols::new(Identifier("b".to_string()), 1),
                TraceCols::new(Identifier("c".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        IntegrityConstraints(vec![ConstraintComprehension(
            ConstraintType::Inline(IntegrityConstraint::new(
                Elem(Identifier("x".to_string())),
                Add(
                    Box::new(Elem(Identifier("a".to_string()))),
                    Box::new(Elem(Identifier("b".to_string()))),
                ),
            )),
            None,
            vec![(
                Identifier("x".to_string()),
                Iterable::Range(Range::new(1, 4)),
            )],
        )]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_comprehension_with_selectors() {
    let source = "
    trace_columns:
        main: [s[2], a, b, c[4]]

    integrity_constraints:
        enf x = a + b when s[0] & s[1] for x in c";

    let expected = Source(vec![
        Trace(Trace {
            main_cols: vec![
                TraceCols::new(Identifier("s".to_string()), 2),
                TraceCols::new(Identifier("a".to_string()), 1),
                TraceCols::new(Identifier("b".to_string()), 1),
                TraceCols::new(Identifier("c".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        IntegrityConstraints(vec![ConstraintComprehension(
            ConstraintType::Inline(IntegrityConstraint::new(
                Elem(Identifier("x".to_string())),
                Add(
                    Box::new(Elem(Identifier("a".to_string()))),
                    Box::new(Elem(Identifier("b".to_string()))),
                ),
            )),
            Some(Mul(
                Box::new(VectorAccess(VectorAccess::new(
                    Identifier("s".to_string()),
                    0,
                ))),
                Box::new(VectorAccess(VectorAccess::new(
                    Identifier("s".to_string()),
                    1,
                ))),
            )),
            vec![(
                Identifier("x".to_string()),
                Iterable::Identifier(Identifier("c".to_string())),
            )],
        )]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_comprehension_with_evaluator_call() {
    let source = "
    ev is_binary(main: [x]):
        enf x^2 = x

    trace_columns:
        main: [a, b, c[4], d[4]]

    integrity_constraints:
        enf is_binary([x]) for x in c";

    let expected = Source(vec![
        EvaluatorFunction(EvaluatorFunction::new(
            Identifier("is_binary".to_string()),
            vec![TraceCols::new(Identifier("x".to_string()), 1)],
            Vec::new(),
            vec![Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    Exp(
                        Box::new(Elem(Identifier("x".to_string()))),
                        Box::new(Const(2)),
                    ),
                    Elem(Identifier("x".to_string())),
                )),
                None,
            )],
        )),
        Trace(Trace {
            main_cols: vec![
                TraceCols::new(Identifier("a".to_string()), 1),
                TraceCols::new(Identifier("b".to_string()), 1),
                TraceCols::new(Identifier("c".to_string()), 4),
                TraceCols::new(Identifier("d".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        IntegrityConstraints(vec![ConstraintComprehension(
            ConstraintType::Evaluator(EvaluatorFunctionCall::new(
                Identifier("is_binary".to_string()),
                vec![vec![TraceCols::new(Identifier("x".to_string()), 1)]],
            )),
            None,
            vec![(
                Identifier("x".to_string()),
                Iterable::Identifier(Identifier("c".to_string())),
            )],
        )]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_comprehension_with_evaluator_and_selectors() {
    let source = "
    ev is_binary(main: [x]):
        enf x^2 = x

    trace_columns:
        main: [s[2], a, b, c[4], d[4]]

    integrity_constraints:
        enf is_binary([x]) when s[0] & s[1] for x in c";

    let expected = Source(vec![
        EvaluatorFunction(EvaluatorFunction::new(
            Identifier("is_binary".to_string()),
            vec![TraceCols::new(Identifier("x".to_string()), 1)],
            Vec::new(),
            vec![Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    Exp(
                        Box::new(Elem(Identifier("x".to_string()))),
                        Box::new(Const(2)),
                    ),
                    Elem(Identifier("x".to_string())),
                )),
                None,
            )],
        )),
        Trace(Trace {
            main_cols: vec![
                TraceCols::new(Identifier("s".to_string()), 2),
                TraceCols::new(Identifier("a".to_string()), 1),
                TraceCols::new(Identifier("b".to_string()), 1),
                TraceCols::new(Identifier("c".to_string()), 4),
                TraceCols::new(Identifier("d".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        IntegrityConstraints(vec![ConstraintComprehension(
            ConstraintType::Evaluator(EvaluatorFunctionCall::new(
                Identifier("is_binary".to_string()),
                vec![vec![TraceCols::new(Identifier("x".to_string()), 1)]],
            )),
            Some(Mul(
                Box::new(VectorAccess(VectorAccess::new(
                    Identifier("s".to_string()),
                    0,
                ))),
                Box::new(VectorAccess(VectorAccess::new(
                    Identifier("s".to_string()),
                    1,
                ))),
            )),
            vec![(
                Identifier("x".to_string()),
                Iterable::Identifier(Identifier("c".to_string())),
            )],
        )]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

// INVALID INTEGRITY CONSTRAINT COMPREHENSION
// ================================================================================================

#[test]
fn err_ic_comprehension_one_member_two_iterables() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    integrity_constraints:
        enf a = c for c in (c, d)";

    let error = Error::ParseError(ParseError::InvalidConstraintComprehension(
        "Number of members and iterables must match".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

#[test]
fn err_ic_comprehension_two_members_one_iterable() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    integrity_constraints:
        enf a = c + d for (c, d) in c";

    let error = Error::ParseError(ParseError::InvalidConstraintComprehension(
        "Number of members and iterables must match".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

// INVALID INTEGRITY CONSTRAINTS
// ================================================================================================

#[test]
fn err_missing_integrity_constraint() {
    let source = "
    integrity_constraints:
        let a = 2^2
        let b = [a, 2 * a]
        let c = [[a - 1, a^2], [b[0], b[1]]]";
    let error = Error::ParseError(ParseError::MissingIntegrityConstraint(
        "Declaration of at least one integrity constraint is required".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

#[test]
fn ic_invalid() {
    let source = "integrity_constraints:
        enf clk' = clk = 1";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn error_invalid_next_usage() {
    let source = "
    integrity_constraints:
        enf clk'' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn err_empty_integrity_constraints() {
    let source = "
    integrity_constraints:
        
    boundary_constraints:
        enf clk.first = 1";
    build_parse_test!(source).expect_unrecognized_token();
}
