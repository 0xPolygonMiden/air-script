use super::{build_parse_test, Identifier, IntegrityConstraint, Source};
use crate::{
    ast::{
        Boundary, BoundaryConstraint, BoundaryStmt, ConstraintType, EvaluatorFunction,
        EvaluatorFunctionCall, Expression::*, IntegrityStmt, Iterable, Range, SourceSection::*,
        TraceBinding, TraceBindingAccess, TraceBindingAccessSize, VectorAccess,
    },
    error::{Error, ParseError},
};

// CONSTRAINT COMPREHENSION
// ================================================================================================

#[test]
fn bc_comprehension_one_iterable_identifier() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    boundary_constraints:
        enf x.first = 0 for x in c";

    let expected = Source(vec![
        Trace(Trace {
            main_cols: vec![
                TraceBinding::new(Identifier("a".to_string()), 1),
                TraceBinding::new(Identifier("b".to_string()), 1),
                TraceBinding::new(Identifier("c".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        BoundaryConstraints(vec![BoundaryStmt::ConstraintComprehension(
            BoundaryConstraint::new(
                TraceBindingAccess::new(Identifier("x".to_string()), 0, TraceBindingAccessSize::Full, 0),
                Boundary::First,
                Const(0),
            ),
            vec![(
                Identifier("x".to_string()),
                Iterable::Identifier(Identifier("c".to_string())),
            )],
        )]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn bc_comprehension_one_iterable_range() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    boundary_constraints:
        enf x.first = 0 for x in (0..4)";

    let expected = Source(vec![
        Trace(Trace {
            main_cols: vec![
                TraceBinding::new(Identifier("a".to_string()), 1),
                TraceBinding::new(Identifier("b".to_string()), 1),
                TraceBinding::new(Identifier("c".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        BoundaryConstraints(vec![BoundaryStmt::ConstraintComprehension(
            BoundaryConstraint::new(
                TraceBindingAccess::new(Identifier("x".to_string()), 0, TraceBindingAccessSize::Full, 0),
                Boundary::First,
                Const(0),
            ),
            vec![(
                Identifier("x".to_string()),
                Iterable::Range(Range::new(0, 4)),
            )],
        )]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn bc_comprehension_one_iterable_slice() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    boundary_constraints:
        enf x.first = 0 for x in c[1..3]";

    let expected = Source(vec![
        Trace(Trace {
            main_cols: vec![
                TraceBinding::new(Identifier("a".to_string()), 1),
                TraceBinding::new(Identifier("b".to_string()), 1),
                TraceBinding::new(Identifier("c".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        BoundaryConstraints(vec![BoundaryStmt::ConstraintComprehension(
            BoundaryConstraint::new(
                TraceBindingAccess::new(Identifier("x".to_string()), 0, TraceBindingAccessSize::Full, 0),
                Boundary::First,
                Const(0),
            ),
            vec![(
                Identifier("x".to_string()),
                Iterable::Slice(Identifier("c".to_string()), Range::new(1, 3)),
            )],
        )]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn bc_comprehension_two_iterable_identifiers() {
    let source = "
    trace_columns:
        main: [a, b, c[4], d[4]]

    boundary_constraints:
        enf x.first = y for (x, y) in (c, d)";

    let expected = Source(vec![
        Trace(Trace {
            main_cols: vec![
                TraceBinding::new(Identifier("a".to_string()), 1),
                TraceBinding::new(Identifier("b".to_string()), 1),
                TraceBinding::new(Identifier("c".to_string()), 4),
                TraceBinding::new(Identifier("d".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        BoundaryConstraints(vec![BoundaryStmt::ConstraintComprehension(
            BoundaryConstraint::new(
                TraceBindingAccess::new(Identifier("x".to_string()), 0, TraceBindingAccessSize::Full, 0),
                Boundary::First,
                Elem(Identifier("y".to_string())),
            ),
            vec![
                (
                    Identifier("x".to_string()),
                    Iterable::Identifier(Identifier("c".to_string())),
                ),
                (
                    Identifier("y".to_string()),
                    Iterable::Identifier(Identifier("d".to_string())),
                ),
            ],
        )]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

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
                TraceBinding::new(Identifier("a".to_string()), 1),
                TraceBinding::new(Identifier("b".to_string()), 1),
                TraceBinding::new(Identifier("c".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        IntegrityConstraints(vec![IntegrityStmt::ConstraintComprehension(
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
                TraceBinding::new(Identifier("a".to_string()), 1),
                TraceBinding::new(Identifier("b".to_string()), 1),
                TraceBinding::new(Identifier("c".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        IntegrityConstraints(vec![IntegrityStmt::ConstraintComprehension(
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
                TraceBinding::new(Identifier("s".to_string()), 2),
                TraceBinding::new(Identifier("a".to_string()), 1),
                TraceBinding::new(Identifier("b".to_string()), 1),
                TraceBinding::new(Identifier("c".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        IntegrityConstraints(vec![IntegrityStmt::ConstraintComprehension(
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
            vec![TraceBinding::new(Identifier("x".to_string()), 1)],
            Vec::new(),
            vec![IntegrityStmt::Constraint(
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
                TraceBinding::new(Identifier("a".to_string()), 1),
                TraceBinding::new(Identifier("b".to_string()), 1),
                TraceBinding::new(Identifier("c".to_string()), 4),
                TraceBinding::new(Identifier("d".to_string()), 4),
            ],
            aux_cols: vec![],
        }),
        IntegrityConstraints(vec![IntegrityStmt::ConstraintComprehension(
            ConstraintType::Evaluator(EvaluatorFunctionCall::new(
                Identifier("is_binary".to_string()),
                vec![vec![TraceBinding::new(Identifier("x".to_string()), 1)]],
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
            vec![IntegrityStmt::Constraint(
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
        IntegrityConstraints(vec![IntegrityStmt::ConstraintComprehension(
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

// INVALID CONSTRAINT COMPREHENSION
// ================================================================================================

#[test]
fn err_bc_comprehension_one_member_two_iterables() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    boundary_constraints:
        enf a.first = c for c in (c, d)";

    let error = Error::ParseError(ParseError::InvalidConstraintComprehension(
        "Number of members and iterables must match".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

#[test]
fn err_bc_comprehension_two_members_one_iterables() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    boundary_constraints:
        enf a.first = c + d for (c, d) in c";

    let error = Error::ParseError(ParseError::InvalidConstraintComprehension(
        "Number of members and iterables must match".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

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
