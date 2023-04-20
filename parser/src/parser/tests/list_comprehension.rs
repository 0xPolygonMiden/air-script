use air_script_core::{Iterable, ListComprehension, Range};

use super::{build_parse_test, Identifier, IntegrityConstraint, Source};
use crate::{
    ast::{
        AccessType, Boundary, BoundaryConstraint, BoundaryStmt, ConstraintType, Expression::*,
        IntegrityStmt, SourceSection::*, SymbolAccess, TraceBinding, VariableBinding,
        VariableValueExpr,
    },
    error::{Error, ParseError},
};

// LIST COMPREHENSION
// ================================================================================================

#[test]
fn bc_one_iterable_identifier_lc() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    boundary_constraints:
        # raise value in the current row to power 7
        let x = [col^7 for col in c]

        enf a.first = x[0] + x[1] + x[2] + x[3]";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 6),
        ]]),
        BoundaryConstraints(vec![
            BoundaryStmt::VariableBinding(VariableBinding::new(
                Identifier("x".to_string()),
                VariableValueExpr::ListComprehension(ListComprehension::new(
                    Exp(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("col".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                        Box::new(Const(7)),
                    ),
                    vec![(
                        Identifier("col".to_string()),
                        Iterable::Identifier(Identifier("c".to_string())),
                    )],
                )),
            )),
            BoundaryStmt::Constraint(BoundaryConstraint::new(
                SymbolAccess::new(Identifier("a".to_string()), AccessType::Default, 0),
                Boundary::First,
                Add(
                    Box::new(Add(
                        Box::new(Add(
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(0),
                                0,
                            ))),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(1),
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Vector(2),
                            0,
                        ))),
                    )),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("x".to_string()),
                        AccessType::Vector(3),
                        0,
                    ))),
                ),
            )),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn bc_identifier_and_range_lc() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    boundary_constraints:
        let x = [2^i * c for (i, c) in (0..3, c)]
        enf a.first = x[0] + x[1] + x[2] + x[3]";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 6),
        ]]),
        BoundaryConstraints(vec![
            BoundaryStmt::VariableBinding(VariableBinding::new(
                Identifier("x".to_string()),
                VariableValueExpr::ListComprehension(ListComprehension::new(
                    Mul(
                        Box::new(Exp(
                            Box::new(Const(2)),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("i".to_string()),
                                AccessType::Default,
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("c".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                    ),
                    vec![
                        (
                            Identifier("i".to_string()),
                            Iterable::Range(Range::new(0, 3)),
                        ),
                        (
                            Identifier("c".to_string()),
                            Iterable::Identifier(Identifier("c".to_string())),
                        ),
                    ],
                )),
            )),
            BoundaryStmt::Constraint(BoundaryConstraint::new(
                SymbolAccess::new(Identifier("a".to_string()), AccessType::Default, 0),
                Boundary::First,
                Add(
                    Box::new(Add(
                        Box::new(Add(
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(0),
                                0,
                            ))),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(1),
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Vector(2),
                            0,
                        ))),
                    )),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("x".to_string()),
                        AccessType::Vector(3),
                        0,
                    ))),
                ),
            )),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn bc_iterable_slice_lc() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    boundary_constraints:
        let x = [c for c in c[0..3]]
        enf a.first = x[0] + x[1] + x[2] + x[3]";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 6),
        ]]),
        BoundaryConstraints(vec![
            BoundaryStmt::VariableBinding(VariableBinding::new(
                Identifier("x".to_string()),
                VariableValueExpr::ListComprehension(ListComprehension::new(
                    SymbolAccess(SymbolAccess::new(
                        Identifier("c".to_string()),
                        AccessType::Default,
                        0,
                    )),
                    vec![(
                        Identifier("c".to_string()),
                        Iterable::Slice(Identifier("c".to_string()), Range::new(0, 3)),
                    )],
                )),
            )),
            BoundaryStmt::Constraint(BoundaryConstraint::new(
                SymbolAccess::new(Identifier("a".to_string()), AccessType::Default, 0),
                Boundary::First,
                Add(
                    Box::new(Add(
                        Box::new(Add(
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(0),
                                0,
                            ))),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(1),
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Vector(2),
                            0,
                        ))),
                    )),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("x".to_string()),
                        AccessType::Vector(3),
                        0,
                    ))),
                ),
            )),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn bc_two_iterable_identifier_lc() {
    let source = "
    trace_columns:
        main: [a, b, c[4], d[4]]

    boundary_constraints:
        let diff = [x - y for (x, y) in (c, d)]
        enf a.first = x[0] + x[1] + x[2] + x[3]";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("d".to_string()), 0, 6, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 10),
        ]]),
        BoundaryConstraints(vec![
            BoundaryStmt::VariableBinding(VariableBinding::new(
                Identifier("diff".to_string()),
                VariableValueExpr::ListComprehension(ListComprehension::new(
                    Sub(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("y".to_string()),
                            AccessType::Default,
                            0,
                        ))),
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
                )),
            )),
            BoundaryStmt::Constraint(BoundaryConstraint::new(
                SymbolAccess::new(Identifier("a".to_string()), AccessType::Default, 0),
                Boundary::First,
                Add(
                    Box::new(Add(
                        Box::new(Add(
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(0),
                                0,
                            ))),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(1),
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Vector(2),
                            0,
                        ))),
                    )),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("x".to_string()),
                        AccessType::Vector(3),
                        0,
                    ))),
                ),
            )),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn bc_multiple_iterables_lc() {
    let source = "
    trace_columns:
        main: [a, b[3], c[4], d[4]]

    boundary_constraints:
        let diff = [w + x - y - z for (w, x, y, z) in (0..3, b, c[0..3], d[0..3])]
        enf a.first = x[0] + x[1] + x[2] + x[3]";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 3),
            TraceBinding::new(Identifier("c".to_string()), 0, 4, 4),
            TraceBinding::new(Identifier("d".to_string()), 0, 8, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 12),
        ]]),
        BoundaryConstraints(vec![
            BoundaryStmt::VariableBinding(VariableBinding::new(
                Identifier("diff".to_string()),
                VariableValueExpr::ListComprehension(ListComprehension::new(
                    Sub(
                        Box::new(Sub(
                            Box::new(Add(
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("w".to_string()),
                                    AccessType::Default,
                                    0,
                                ))),
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("x".to_string()),
                                    AccessType::Default,
                                    0,
                                ))),
                            )),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("y".to_string()),
                                AccessType::Default,
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("z".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                    ),
                    vec![
                        (
                            Identifier("w".to_string()),
                            Iterable::Range(Range::new(0, 3)),
                        ),
                        (
                            Identifier("x".to_string()),
                            Iterable::Identifier(Identifier("b".to_string())),
                        ),
                        (
                            Identifier("y".to_string()),
                            Iterable::Slice(Identifier("c".to_string()), Range::new(0, 3)),
                        ),
                        (
                            Identifier("z".to_string()),
                            Iterable::Slice(Identifier("d".to_string()), Range::new(0, 3)),
                        ),
                    ],
                )),
            )),
            BoundaryStmt::Constraint(BoundaryConstraint::new(
                SymbolAccess::new(Identifier("a".to_string()), AccessType::Default, 0),
                Boundary::First,
                Add(
                    Box::new(Add(
                        Box::new(Add(
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(0),
                                0,
                            ))),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(1),
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Vector(2),
                            0,
                        ))),
                    )),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("x".to_string()),
                        AccessType::Vector(3),
                        0,
                    ))),
                ),
            )),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_one_iterable_identifier_lc() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    integrity_constraints:
        # raise value in the current row to power 7
        let x = [col^7 for col in c]

        # raise value in the next row to power 7
        let y = [col'^7 for col in c]
        enf a = x[0] + x[1] + x[2] + x[3]";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 6),
        ]]),
        IntegrityConstraints(vec![
            IntegrityStmt::VariableBinding(VariableBinding::new(
                Identifier("x".to_string()),
                VariableValueExpr::ListComprehension(ListComprehension::new(
                    Exp(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("col".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                        Box::new(Const(7)),
                    ),
                    vec![(
                        Identifier("col".to_string()),
                        Iterable::Identifier(Identifier("c".to_string())),
                    )],
                )),
            )),
            IntegrityStmt::VariableBinding(VariableBinding::new(
                Identifier("y".to_string()),
                VariableValueExpr::ListComprehension(ListComprehension::new(
                    Exp(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("col".to_string()),
                            AccessType::Default,
                            1,
                        ))),
                        Box::new(Const(7)),
                    ),
                    vec![(
                        Identifier("col".to_string()),
                        Iterable::Identifier(Identifier("c".to_string())),
                    )],
                )),
            )),
            IntegrityStmt::Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Default,
                        0,
                    )),
                    Add(
                        Box::new(Add(
                            Box::new(Add(
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("x".to_string()),
                                    AccessType::Vector(0),
                                    0,
                                ))),
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("x".to_string()),
                                    AccessType::Vector(1),
                                    0,
                                ))),
                            )),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(2),
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Vector(3),
                            0,
                        ))),
                    ),
                )),
                None,
            ),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_iterable_identifier_range_lc() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    integrity_constraints:
        let x = [2^i * c for (i, c) in (0..3, c)]
        enf a = x[0] + x[1] + x[2] + x[3]";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 6),
        ]]),
        IntegrityConstraints(vec![
            IntegrityStmt::VariableBinding(VariableBinding::new(
                Identifier("x".to_string()),
                VariableValueExpr::ListComprehension(ListComprehension::new(
                    Mul(
                        Box::new(Exp(
                            Box::new(Const(2)),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("i".to_string()),
                                AccessType::Default,
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("c".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                    ),
                    vec![
                        (
                            Identifier("i".to_string()),
                            Iterable::Range(Range::new(0, 3)),
                        ),
                        (
                            Identifier("c".to_string()),
                            Iterable::Identifier(Identifier("c".to_string())),
                        ),
                    ],
                )),
            )),
            IntegrityStmt::Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Default,
                        0,
                    )),
                    Add(
                        Box::new(Add(
                            Box::new(Add(
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("x".to_string()),
                                    AccessType::Vector(0),
                                    0,
                                ))),
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("x".to_string()),
                                    AccessType::Vector(1),
                                    0,
                                ))),
                            )),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(2),
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Vector(3),
                            0,
                        ))),
                    ),
                )),
                None,
            ),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_iterable_slice_lc() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    integrity_constraints:
        let x = [c for c in c[0..3]]
        enf a = x[0] + x[1] + x[2] + x[3]";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 6),
        ]]),
        IntegrityConstraints(vec![
            IntegrityStmt::VariableBinding(VariableBinding::new(
                Identifier("x".to_string()),
                VariableValueExpr::ListComprehension(ListComprehension::new(
                    SymbolAccess(SymbolAccess::new(
                        Identifier("c".to_string()),
                        AccessType::Default,
                        0,
                    )),
                    vec![(
                        Identifier("c".to_string()),
                        Iterable::Slice(Identifier("c".to_string()), Range::new(0, 3)),
                    )],
                )),
            )),
            IntegrityStmt::Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Default,
                        0,
                    )),
                    Add(
                        Box::new(Add(
                            Box::new(Add(
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("x".to_string()),
                                    AccessType::Vector(0),
                                    0,
                                ))),
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("x".to_string()),
                                    AccessType::Vector(1),
                                    0,
                                ))),
                            )),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(2),
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Vector(3),
                            0,
                        ))),
                    ),
                )),
                None,
            ),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_two_iterable_identifier_lc() {
    let source = "
    trace_columns:
        main: [a, b, c[4], d[4]]

    integrity_constraints:
        let diff = [x - y for (x, y) in (c, d)]
        enf a = x[0] + x[1] + x[2] + x[3]";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("d".to_string()), 0, 6, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 10),
        ]]),
        IntegrityConstraints(vec![
            IntegrityStmt::VariableBinding(VariableBinding::new(
                Identifier("diff".to_string()),
                VariableValueExpr::ListComprehension(ListComprehension::new(
                    Sub(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("y".to_string()),
                            AccessType::Default,
                            0,
                        ))),
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
                )),
            )),
            IntegrityStmt::Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Default,
                        0,
                    )),
                    Add(
                        Box::new(Add(
                            Box::new(Add(
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("x".to_string()),
                                    AccessType::Vector(0),
                                    0,
                                ))),
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("x".to_string()),
                                    AccessType::Vector(1),
                                    0,
                                ))),
                            )),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(2),
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Vector(3),
                            0,
                        ))),
                    ),
                )),
                None,
            ),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn ic_multiple_iterables_lc() {
    let source = "
    trace_columns:
        main: [a, b[3], c[4], d[4]]

    integrity_constraints:
        let diff = [w + x - y - z for (w, x, y, z) in (0..3, b, c[0..3], d[0..3])]
        enf a = x[0] + x[1] + x[2] + x[3]";

    let expected = Source(vec![
        Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 3),
            TraceBinding::new(Identifier("c".to_string()), 0, 4, 4),
            TraceBinding::new(Identifier("d".to_string()), 0, 8, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 12),
        ]]),
        IntegrityConstraints(vec![
            IntegrityStmt::VariableBinding(VariableBinding::new(
                Identifier("diff".to_string()),
                VariableValueExpr::ListComprehension(ListComprehension::new(
                    Sub(
                        Box::new(Sub(
                            Box::new(Add(
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("w".to_string()),
                                    AccessType::Default,
                                    0,
                                ))),
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("x".to_string()),
                                    AccessType::Default,
                                    0,
                                ))),
                            )),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("y".to_string()),
                                AccessType::Default,
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("z".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                    ),
                    vec![
                        (
                            Identifier("w".to_string()),
                            Iterable::Range(Range::new(0, 3)),
                        ),
                        (
                            Identifier("x".to_string()),
                            Iterable::Identifier(Identifier("b".to_string())),
                        ),
                        (
                            Identifier("y".to_string()),
                            Iterable::Slice(Identifier("c".to_string()), Range::new(0, 3)),
                        ),
                        (
                            Identifier("z".to_string()),
                            Iterable::Slice(Identifier("d".to_string()), Range::new(0, 3)),
                        ),
                    ],
                )),
            )),
            IntegrityStmt::Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Default,
                        0,
                    )),
                    Add(
                        Box::new(Add(
                            Box::new(Add(
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("x".to_string()),
                                    AccessType::Vector(0),
                                    0,
                                ))),
                                Box::new(SymbolAccess(SymbolAccess::new(
                                    Identifier("x".to_string()),
                                    AccessType::Vector(1),
                                    0,
                                ))),
                            )),
                            Box::new(SymbolAccess(SymbolAccess::new(
                                Identifier("x".to_string()),
                                AccessType::Vector(2),
                                0,
                            ))),
                        )),
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Vector(3),
                            0,
                        ))),
                    ),
                )),
                None,
            ),
        ]),
    ]);

    build_parse_test!(source).expect_ast(expected);
}

// INVALID LIST COMPREHENSION
// ================================================================================================

#[test]
fn err_bc_lc_one_member_two_iterables() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    boundary_constraints:
        let x = [c for c in (c, d)]
        enf a.first = x";

    let error = Error::ParseError(ParseError::InvalidListComprehension(
        "Number of members and iterables must match".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

#[test]
fn err_bc_lc_two_members_one_iterables() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    boundary_constraints:
        let x = [c + d for (c, d) in c]
        enf a.first = x";

    let error = Error::ParseError(ParseError::InvalidListComprehension(
        "Number of members and iterables must match".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

#[test]
fn err_ic_lc_one_member_two_iterables() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    integrity_constraints:
        let x = [c for c in (c, d)]
        enf a = x";

    let error = Error::ParseError(ParseError::InvalidListComprehension(
        "Number of members and iterables must match".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

#[test]
fn err_ic_lc_two_members_one_iterable() {
    let source = "
    trace_columns:
        main: [a, b, c[4]]

    integrity_constraints:
        let x = [c + d for (c, d) in c]
        enf a = x";

    let error = Error::ParseError(ParseError::InvalidListComprehension(
        "Number of members and iterables must match".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}
