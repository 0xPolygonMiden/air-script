use super::{
    build_parse_test, AccessType, Boundary, BoundaryConstraint, Identifier, Iterable, Range,
    Source, SourceSection, SymbolAccess, TraceBinding,
};
use crate::{
    ast::{
        BoundaryStmt::*, ConstantBinding, ConstantValueExpr::*, Expression::*, PublicInput,
        VariableBinding, VariableValueExpr,
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
            SymbolAccess::new(Identifier("clk".to_string()), AccessType::Default, 0),
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
            SymbolAccess::new(Identifier("clk".to_string()), AccessType::Default, 0),
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
            SymbolAccess::new(Identifier("clk".to_string()), AccessType::Default, 0),
            Boundary::First,
            Const(0),
        )),
        Constraint(BoundaryConstraint::new(
            SymbolAccess::new(Identifier("clk".to_string()), AccessType::Default, 0),
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
            SymbolAccess::new(Identifier("clk".to_string()), AccessType::Default, 0),
            Boundary::First,
            SymbolAccess(SymbolAccess::new(
                Identifier("a".to_string()),
                AccessType::Vector(0),
                0,
            )),
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
            SymbolAccess::new(Identifier("clk".to_string()), AccessType::Default, 0),
            Boundary::First,
            Add(
                Box::new(Add(
                    Box::new(Const(5)),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Vector(3),
                        0,
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
        SourceSection::Constant(ConstantBinding::new(Identifier("A".to_string()), Scalar(1))),
        SourceSection::Constant(ConstantBinding::new(
            Identifier("B".to_string()),
            Vector(vec![0, 1]),
        )),
        SourceSection::Constant(ConstantBinding::new(
            Identifier("C".to_string()),
            Matrix(vec![vec![0, 1], vec![1, 0]]),
        )),
        SourceSection::BoundaryConstraints(vec![Constraint(BoundaryConstraint::new(
            SymbolAccess::new(Identifier("clk".to_string()), AccessType::Default, 0),
            Boundary::First,
            Sub(
                Box::new(Add(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("A".to_string()),
                        AccessType::Default,
                        0,
                    ))),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("B".to_string()),
                        AccessType::Vector(1),
                        0,
                    ))),
                )),
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("C".to_string()),
                    AccessType::Matrix(0, 1),
                    0,
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
        VariableBinding(VariableBinding::new(
            Identifier("a".to_string()),
            VariableValueExpr::Scalar(Exp(Box::new(Const(2)), Box::new(Const(2)))),
        )),
        VariableBinding(VariableBinding::new(
            Identifier("b".to_string()),
            VariableValueExpr::Vector(vec![
                SymbolAccess(SymbolAccess::new(
                    Identifier("a".to_string()),
                    AccessType::Default,
                    0,
                )),
                Mul(
                    Box::new(Const(2)),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Default,
                        0,
                    ))),
                ),
            ]),
        )),
        VariableBinding(VariableBinding::new(
            Identifier("c".to_string()),
            VariableValueExpr::Matrix(vec![
                vec![
                    Sub(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("a".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                        Box::new(Const(1)),
                    ),
                    Exp(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("a".to_string()),
                            AccessType::Default,
                            0,
                        ))),
                        Box::new(Const(2)),
                    ),
                ],
                vec![
                    SymbolAccess(SymbolAccess::new(
                        Identifier("b".to_string()),
                        AccessType::Vector(0),
                        0,
                    )),
                    SymbolAccess(SymbolAccess::new(
                        Identifier("b".to_string()),
                        AccessType::Vector(1),
                        0,
                    )),
                ],
            ]),
        )),
        Constraint(BoundaryConstraint::new(
            SymbolAccess::new(Identifier("clk".to_string()), AccessType::Default, 0),
            Boundary::First,
            Add(
                Box::new(Add(
                    Box::new(Const(5)),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Vector(3),
                        0,
                    ))),
                )),
                Box::new(Const(6)),
            ),
        )),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

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
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 6),
        ]]),
        SourceSection::BoundaryConstraints(vec![ConstraintComprehension(
            BoundaryConstraint::new(
                SymbolAccess::new(Identifier("x".to_string()), AccessType::Default, 0),
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
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 6),
        ]]),
        SourceSection::BoundaryConstraints(vec![ConstraintComprehension(
            BoundaryConstraint::new(
                SymbolAccess::new(Identifier("x".to_string()), AccessType::Default, 0),
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
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 6),
        ]]),
        SourceSection::BoundaryConstraints(vec![ConstraintComprehension(
            BoundaryConstraint::new(
                SymbolAccess::new(Identifier("x".to_string()), AccessType::Default, 0),
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
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("d".to_string()), 0, 6, 4),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 10),
        ]]),
        SourceSection::BoundaryConstraints(vec![ConstraintComprehension(
            BoundaryConstraint::new(
                SymbolAccess::new(Identifier("x".to_string()), AccessType::Default, 0),
                Boundary::First,
                SymbolAccess(SymbolAccess::new(
                    Identifier("y".to_string()),
                    AccessType::Default,
                    0,
                )),
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

// INVALID BOUNDARY CONSTRAINT COMPREHENSION
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

// INVALID BOUNDARY CONSTRAINTS
// ================================================================================================

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
