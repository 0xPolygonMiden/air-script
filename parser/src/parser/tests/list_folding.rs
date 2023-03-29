use air_script_core::{Iterable, ListComprehension, ListFoldingType, ListFoldingValueType, Range};

use super::{build_parse_test, Identifier, IntegrityConstraint, Source};
use crate::{
    ast::{
        Boundary, BoundaryConstraint, BoundaryStmt, ConstraintType, Expression::*, IntegrityStmt,
        SourceSection::*, TraceBinding, TraceBindingAccess, TraceBindingAccessSize, Variable,
        VariableType, VectorAccess,
    },
    error::{Error, ParseError},
};

// LIST FOLDING
// ================================================================================================

#[test]
fn identifier_lf() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]
    boundary_constraints:
        let x = sum(c)
        let y = prod(c)
        enf a.first = x + y";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
        ]]),
        BoundaryConstraints(vec![
            BoundaryStmt::Variable(Variable::new(
                Identifier("x".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Sum(
                    ListFoldingValueType::Identifier(Identifier("c".to_string())),
                ))),
            )),
            BoundaryStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Prod(
                    ListFoldingValueType::Identifier(Identifier("c".to_string())),
                ))),
            )),
            BoundaryStmt::Constraint(BoundaryConstraint::new(
                TraceBindingAccess::new(
                    Identifier("a".to_string()),
                    0,
                    TraceBindingAccessSize::Full,
                    0,
                ),
                Boundary::First,
                Add(
                    Box::new(Elem(Identifier("x".to_string()))),
                    Box::new(Elem(Identifier("y".to_string()))),
                ),
            )),
        ]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn vector_lf() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]
    boundary_constraints:
        let x = sum([a, b, c[0]])
        let y = prod([a, b, c[0]])
        enf a.first = x + y";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
        ]]),
        BoundaryConstraints(vec![
            BoundaryStmt::Variable(Variable::new(
                Identifier("x".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Sum(
                    ListFoldingValueType::Vector(vec![
                        Elem(Identifier("a".to_string())),
                        Elem(Identifier("b".to_string())),
                        VectorAccess(VectorAccess::new(Identifier("c".to_string()), 0)),
                    ]),
                ))),
            )),
            BoundaryStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Prod(
                    ListFoldingValueType::Vector(vec![
                        Elem(Identifier("a".to_string())),
                        Elem(Identifier("b".to_string())),
                        VectorAccess(VectorAccess::new(Identifier("c".to_string()), 0)),
                    ]),
                ))),
            )),
            BoundaryStmt::Constraint(BoundaryConstraint::new(
                TraceBindingAccess::new(
                    Identifier("a".to_string()),
                    0,
                    TraceBindingAccessSize::Full,
                    0,
                ),
                Boundary::First,
                Add(
                    Box::new(Elem(Identifier("x".to_string()))),
                    Box::new(Elem(Identifier("y".to_string()))),
                ),
            )),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn bc_one_iterable_identifier_lf() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]
    boundary_constraints:
        let x = sum([col^7 for col in c])
        let y = prod([col^7 for col in c])
        enf a.first = x + y";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
        ]]),
        BoundaryConstraints(vec![
            BoundaryStmt::Variable(Variable::new(
                Identifier("x".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Sum(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Exp(
                            Box::new(Elem(Identifier("col".to_string()))),
                            Box::new(Const(7)),
                        ),
                        vec![(
                            Identifier("col".to_string()),
                            Iterable::Identifier(Identifier("c".to_string())),
                        )],
                    )),
                ))),
            )),
            BoundaryStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Prod(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Exp(
                            Box::new(Elem(Identifier("col".to_string()))),
                            Box::new(Const(7)),
                        ),
                        vec![(
                            Identifier("col".to_string()),
                            Iterable::Identifier(Identifier("c".to_string())),
                        )],
                    )),
                ))),
            )),
            BoundaryStmt::Constraint(BoundaryConstraint::new(
                TraceBindingAccess::new(
                    Identifier("a".to_string()),
                    0,
                    TraceBindingAccessSize::Full,
                    0,
                ),
                Boundary::First,
                Add(
                    Box::new(Elem(Identifier("x".to_string()))),
                    Box::new(Elem(Identifier("y".to_string()))),
                ),
            )),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn bc_two_iterable_identifier_lf() {
    let source = "
    trace_columns:
        main: [a, b, c[4], d[4]]
    boundary_constraints:
        let x = sum([c * d for (c, d) in (c, d)])
        let y = prod([c + d for (c, d) in (c, d)])
        enf a.first = x + y";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("d".to_string()), 0, 6, 4),
        ]]),
        BoundaryConstraints(vec![
            BoundaryStmt::Variable(Variable::new(
                Identifier("x".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Sum(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Mul(
                            Box::new(Elem(Identifier("c".to_string()))),
                            Box::new(Elem(Identifier("d".to_string()))),
                        ),
                        vec![
                            (
                                Identifier("c".to_string()),
                                Iterable::Identifier(Identifier("c".to_string())),
                            ),
                            (
                                Identifier("d".to_string()),
                                Iterable::Identifier(Identifier("d".to_string())),
                            ),
                        ],
                    )),
                ))),
            )),
            BoundaryStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Prod(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Add(
                            Box::new(Elem(Identifier("c".to_string()))),
                            Box::new(Elem(Identifier("d".to_string()))),
                        ),
                        vec![
                            (
                                Identifier("c".to_string()),
                                Iterable::Identifier(Identifier("c".to_string())),
                            ),
                            (
                                Identifier("d".to_string()),
                                Iterable::Identifier(Identifier("d".to_string())),
                            ),
                        ],
                    )),
                ))),
            )),
            BoundaryStmt::Constraint(BoundaryConstraint::new(
                TraceBindingAccess::new(
                    Identifier("a".to_string()),
                    0,
                    TraceBindingAccessSize::Full,
                    0,
                ),
                Boundary::First,
                Add(
                    Box::new(Elem(Identifier("x".to_string()))),
                    Box::new(Elem(Identifier("y".to_string()))),
                ),
            )),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn bc_two_iterables_identifier_range_lf() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]
    boundary_constraints:
        let x = sum([i * c for (i, c) in (0..4, c)])
        let y = prod([i + c for (i, c) in (0..4, c)])
        enf a.first = x + y";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
        ]]),
        BoundaryConstraints(vec![
            BoundaryStmt::Variable(Variable::new(
                Identifier("x".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Sum(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Mul(
                            Box::new(Elem(Identifier("i".to_string()))),
                            Box::new(Elem(Identifier("c".to_string()))),
                        ),
                        vec![
                            (
                                Identifier("i".to_string()),
                                Iterable::Range(Range::new(0, 4)),
                            ),
                            (
                                Identifier("c".to_string()),
                                Iterable::Identifier(Identifier("c".to_string())),
                            ),
                        ],
                    )),
                ))),
            )),
            BoundaryStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Prod(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Add(
                            Box::new(Elem(Identifier("i".to_string()))),
                            Box::new(Elem(Identifier("c".to_string()))),
                        ),
                        vec![
                            (
                                Identifier("i".to_string()),
                                Iterable::Range(Range::new(0, 4)),
                            ),
                            (
                                Identifier("c".to_string()),
                                Iterable::Identifier(Identifier("c".to_string())),
                            ),
                        ],
                    )),
                ))),
            )),
            BoundaryStmt::Constraint(BoundaryConstraint::new(
                TraceBindingAccess::new(
                    Identifier("a".to_string()),
                    0,
                    TraceBindingAccessSize::Full,
                    0,
                ),
                Boundary::First,
                Add(
                    Box::new(Elem(Identifier("x".to_string()))),
                    Box::new(Elem(Identifier("y".to_string()))),
                ),
            )),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_one_iterable_identifier_lf() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]
    integrity_constraints:
        let x = sum([col^7 for col in c])
        let y = prod([col^7 for col in c])
        enf a = x + y";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
        ]]),
        IntegrityConstraints(vec![
            IntegrityStmt::Variable(Variable::new(
                Identifier("x".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Sum(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Exp(
                            Box::new(Elem(Identifier("col".to_string()))),
                            Box::new(Const(7)),
                        ),
                        vec![(
                            Identifier("col".to_string()),
                            Iterable::Identifier(Identifier("c".to_string())),
                        )],
                    )),
                ))),
            )),
            IntegrityStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Prod(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Exp(
                            Box::new(Elem(Identifier("col".to_string()))),
                            Box::new(Const(7)),
                        ),
                        vec![(
                            Identifier("col".to_string()),
                            Iterable::Identifier(Identifier("c".to_string())),
                        )],
                    )),
                ))),
            )),
            IntegrityStmt::Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    Elem(Identifier("a".to_string())),
                    Add(
                        Box::new(Elem(Identifier("x".to_string()))),
                        Box::new(Elem(Identifier("y".to_string()))),
                    ),
                )),
                None,
            ),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_two_iterable_identifier_lf() {
    let source = "
    trace_columns:
        main: [a, b, c[4], d[4]]
    integrity_constraints:
        let x = sum([c * d for (c, d) in (c, d)])
        let y = prod([c + d for (c, d) in (c, d)])
        enf a = x + y";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("d".to_string()), 0, 6, 4),
        ]]),
        IntegrityConstraints(vec![
            IntegrityStmt::Variable(Variable::new(
                Identifier("x".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Sum(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Mul(
                            Box::new(Elem(Identifier("c".to_string()))),
                            Box::new(Elem(Identifier("d".to_string()))),
                        ),
                        vec![
                            (
                                Identifier("c".to_string()),
                                Iterable::Identifier(Identifier("c".to_string())),
                            ),
                            (
                                Identifier("d".to_string()),
                                Iterable::Identifier(Identifier("d".to_string())),
                            ),
                        ],
                    )),
                ))),
            )),
            IntegrityStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Prod(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Add(
                            Box::new(Elem(Identifier("c".to_string()))),
                            Box::new(Elem(Identifier("d".to_string()))),
                        ),
                        vec![
                            (
                                Identifier("c".to_string()),
                                Iterable::Identifier(Identifier("c".to_string())),
                            ),
                            (
                                Identifier("d".to_string()),
                                Iterable::Identifier(Identifier("d".to_string())),
                            ),
                        ],
                    )),
                ))),
            )),
            IntegrityStmt::Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    Elem(Identifier("a".to_string())),
                    Add(
                        Box::new(Elem(Identifier("x".to_string()))),
                        Box::new(Elem(Identifier("y".to_string()))),
                    ),
                )),
                None,
            ),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_two_iterables_identifier_range_lf() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]
    integrity_constraints:
        let x = sum([i * c for (i, c) in (0..4, c)])
        let y = prod([i + c for (i, c) in (0..4, c)])
        enf a = x + y";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
        ]]),
        IntegrityConstraints(vec![
            IntegrityStmt::Variable(Variable::new(
                Identifier("x".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Sum(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Mul(
                            Box::new(Elem(Identifier("i".to_string()))),
                            Box::new(Elem(Identifier("c".to_string()))),
                        ),
                        vec![
                            (
                                Identifier("i".to_string()),
                                Iterable::Range(Range::new(0, 4)),
                            ),
                            (
                                Identifier("c".to_string()),
                                Iterable::Identifier(Identifier("c".to_string())),
                            ),
                        ],
                    )),
                ))),
            )),
            IntegrityStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Prod(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Add(
                            Box::new(Elem(Identifier("i".to_string()))),
                            Box::new(Elem(Identifier("c".to_string()))),
                        ),
                        vec![
                            (
                                Identifier("i".to_string()),
                                Iterable::Range(Range::new(0, 4)),
                            ),
                            (
                                Identifier("c".to_string()),
                                Iterable::Identifier(Identifier("c".to_string())),
                            ),
                        ],
                    )),
                ))),
            )),
            IntegrityStmt::Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    Elem(Identifier("a".to_string())),
                    Add(
                        Box::new(Elem(Identifier("x".to_string()))),
                        Box::new(Elem(Identifier("y".to_string()))),
                    ),
                )),
                None,
            ),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_three_iterables_slice_identifier_range_lf() {
    let source = "
    trace_columns:
        main: [a, b[6], c[4]]
    integrity_constraints:
        let x = sum([m * n * i for (m, n, i) in (b[1..5], c, 0..4)])
        let y = sum([m * n * i for (m, n, i) in (b[1..5], c, 0..4)])
        enf a = x + y";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 6),
            TraceBinding::new(Identifier("c".to_string()), 0, 7, 4),
        ]]),
        IntegrityConstraints(vec![
            IntegrityStmt::Variable(Variable::new(
                Identifier("x".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Sum(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Mul(
                            Box::new(Mul(
                                Box::new(Elem(Identifier("m".to_string()))),
                                Box::new(Elem(Identifier("n".to_string()))),
                            )),
                            Box::new(Elem(Identifier("i".to_string()))),
                        ),
                        vec![
                            (
                                Identifier("m".to_string()),
                                Iterable::Slice(Identifier("b".to_string()), Range::new(1, 5)),
                            ),
                            (
                                Identifier("n".to_string()),
                                Iterable::Identifier(Identifier("c".to_string())),
                            ),
                            (
                                Identifier("i".to_string()),
                                Iterable::Range(Range::new(0, 4)),
                            ),
                        ],
                    )),
                ))),
            )),
            IntegrityStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(ListFolding(ListFoldingType::Sum(
                    ListFoldingValueType::ListComprehension(ListComprehension::new(
                        Mul(
                            Box::new(Mul(
                                Box::new(Elem(Identifier("m".to_string()))),
                                Box::new(Elem(Identifier("n".to_string()))),
                            )),
                            Box::new(Elem(Identifier("i".to_string()))),
                        ),
                        vec![
                            (
                                Identifier("m".to_string()),
                                Iterable::Slice(Identifier("b".to_string()), Range::new(1, 5)),
                            ),
                            (
                                Identifier("n".to_string()),
                                Iterable::Identifier(Identifier("c".to_string())),
                            ),
                            (
                                Identifier("i".to_string()),
                                Iterable::Range(Range::new(0, 4)),
                            ),
                        ],
                    )),
                ))),
            )),
            IntegrityStmt::Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    Elem(Identifier("a".to_string())),
                    Add(
                        Box::new(Elem(Identifier("x".to_string()))),
                        Box::new(Elem(Identifier("y".to_string()))),
                    ),
                )),
                None,
            ),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

// INVALID LIST FOLDING
// ================================================================================================

#[test]
fn err_ic_lf_single_members_double_iterables() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    integrity_constraints:
        let x = sum([c for c in (c, d)])
        enf a = x";

    let error = Error::ParseError(ParseError::InvalidListComprehension(
        "Number of members and iterables must match".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}
