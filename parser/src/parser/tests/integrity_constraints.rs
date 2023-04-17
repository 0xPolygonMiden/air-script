use super::{
    build_parse_test, Identifier, IntegrityConstraint, Iterable, Range, Source, SourceSection,
    TraceBinding,
};
use crate::{
    ast::{
        AccessType, ConstantBinding, ConstantValueExpr::*, ConstraintType, EvaluatorFunction,
        EvaluatorFunctionCall, Expression::*, IntegrityStmt::*, SymbolAccess, TraceAccess,
        TraceBindingAccess, TraceBindingAccessSize, VariableBinding, VariableValueExpr,
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
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("clk".to_string()),
                    AccessType::Default,
                ))),
                Box::new(Const(1)),
            ),
        )),
        None,
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn integrity_constraints_invalid() {
    let source = "integrity_constraints:
        enf clk' = clk = 1";
    build_parse_test!(source).expect_unrecognized_token();
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
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                    ))),
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
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                    ))),
                ),
                Const(1),
            )),
            None,
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn integrity_constraint_with_periodic_col() {
    let source = "
    integrity_constraints:
        enf k0 + b = 0";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        ConstraintType::Inline(IntegrityConstraint::new(
            Add(
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("k0".to_string()),
                    AccessType::Default,
                ))),
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("b".to_string()),
                    AccessType::Default,
                ))),
            ),
            Const(0),
        )),
        None,
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn integrity_constraint_with_random_value() {
    let source = "
    integrity_constraints:
        enf a + $rand[1] = 0";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![Constraint(
        ConstraintType::Inline(IntegrityConstraint::new(
            Add(
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("a".to_string()),
                    AccessType::Default,
                ))),
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("$rand".to_string()),
                    AccessType::Vector(1),
                ))),
            ),
            Const(0),
        )),
        None,
    )])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn integrity_constraint_with_constants() {
    let source = "
        const A = 0
        const B = [0, 1]
        const C = [[0, 1], [1, 0]]
    integrity_constraints:
        enf clk + A = B[1] + C[1][1]";
    let expected = Source(vec![
        SourceSection::Constant(ConstantBinding::new(Identifier("A".to_string()), Scalar(0))),
        SourceSection::Constant(ConstantBinding::new(
            Identifier("B".to_string()),
            Vector(vec![0, 1]),
        )),
        SourceSection::Constant(ConstantBinding::new(
            Identifier("C".to_string()),
            Matrix(vec![vec![0, 1], vec![1, 0]]),
        )),
        SourceSection::IntegrityConstraints(vec![Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                Add(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                    ))),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("A".to_string()),
                        AccessType::Default,
                    ))),
                ),
                Add(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("B".to_string()),
                        AccessType::Vector(1),
                    ))),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("C".to_string()),
                        AccessType::Matrix(1, 1),
                    ))),
                ),
            )),
            None,
        )]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn integrity_constraint_with_variables() {
    let source = "
    integrity_constraints:
        let a = 2^2
        let b = [a, 2 * a]
        let c = [[a - 1, a^2], [b[0], b[1]]]
        enf clk + a = b[1] + c[1][1]";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![
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
                )),
                Mul(
                    Box::new(Const(2)),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Default,
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
                        ))),
                        Box::new(Const(1)),
                    ),
                    Exp(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("a".to_string()),
                            AccessType::Default,
                        ))),
                        Box::new(Const(2)),
                    ),
                ],
                vec![
                    SymbolAccess(SymbolAccess::new(
                        Identifier("b".to_string()),
                        AccessType::Vector(0),
                    )),
                    SymbolAccess(SymbolAccess::new(
                        Identifier("b".to_string()),
                        AccessType::Vector(1),
                    )),
                ],
            ]),
        )),
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                Add(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                    ))),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Default,
                    ))),
                ),
                Add(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("b".to_string()),
                        AccessType::Vector(1),
                    ))),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("c".to_string()),
                        AccessType::Matrix(1, 1),
                    ))),
                ),
            )),
            None,
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn integrity_constraint_with_indexed_trace_access() {
    let source = "
    integrity_constraints:
        enf $main[0]' = $main[1] + 1
        enf $aux[0]' - $aux[1] = 1";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                TraceAccess(TraceAccess::new(0, 0, 1, 1)),
                Add(
                    Box::new(TraceAccess(TraceAccess::new(0, 1, 1, 0))),
                    Box::new(Const(1)),
                ),
            )),
            None,
        ),
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                Sub(
                    Box::new(TraceAccess(TraceAccess::new(1, 0, 1, 1))),
                    Box::new(TraceAccess(TraceAccess::new(1, 1, 1, 0))),
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
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
        ]]),
        SourceSection::IntegrityConstraints(vec![ConstraintComprehension(
            ConstraintType::Inline(IntegrityConstraint::new(
                SymbolAccess(SymbolAccess::new(
                    Identifier("x".to_string()),
                    AccessType::Default,
                )),
                Add(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Default,
                    ))),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("b".to_string()),
                        AccessType::Default,
                    ))),
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
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
        ]]),
        SourceSection::IntegrityConstraints(vec![ConstraintComprehension(
            ConstraintType::Inline(IntegrityConstraint::new(
                SymbolAccess(SymbolAccess::new(
                    Identifier("x".to_string()),
                    AccessType::Default,
                )),
                Add(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Default,
                    ))),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("b".to_string()),
                        AccessType::Default,
                    ))),
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
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("s".to_string()), 0, 0, 2),
            TraceBinding::new(Identifier("a".to_string()), 0, 2, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 3, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 4, 4),
        ]]),
        SourceSection::IntegrityConstraints(vec![ConstraintComprehension(
            ConstraintType::Inline(IntegrityConstraint::new(
                SymbolAccess(SymbolAccess::new(
                    Identifier("x".to_string()),
                    AccessType::Default,
                )),
                Add(
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("a".to_string()),
                        AccessType::Default,
                    ))),
                    Box::new(SymbolAccess(SymbolAccess::new(
                        Identifier("b".to_string()),
                        AccessType::Default,
                    ))),
                ),
            )),
            Some(Mul(
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("s".to_string()),
                    AccessType::Vector(0),
                ))),
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("s".to_string()),
                    AccessType::Vector(1),
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
        SourceSection::EvaluatorFunction(EvaluatorFunction::new(
            Identifier("is_binary".to_string()),
            vec![TraceBinding::new(Identifier("x".to_string()), 0, 0, 1)],
            vec![Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    Exp(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Default,
                        ))),
                        Box::new(Const(2)),
                    ),
                    SymbolAccess(SymbolAccess::new(
                        Identifier("x".to_string()),
                        AccessType::Default,
                    )),
                )),
                None,
            )],
        )),
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 4),
            TraceBinding::new(Identifier("d".to_string()), 0, 6, 4),
        ]]),
        SourceSection::IntegrityConstraints(vec![ConstraintComprehension(
            ConstraintType::Evaluator(EvaluatorFunctionCall::new(
                Identifier("is_binary".to_string()),
                vec![vec![TraceBindingAccess::new(
                    Identifier("x".to_string()),
                    0,
                    TraceBindingAccessSize::Full,
                    0,
                )]],
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
        SourceSection::EvaluatorFunction(EvaluatorFunction::new(
            Identifier("is_binary".to_string()),
            vec![TraceBinding::new(Identifier("x".to_string()), 0, 0, 1)],
            vec![Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    Exp(
                        Box::new(SymbolAccess(SymbolAccess::new(
                            Identifier("x".to_string()),
                            AccessType::Default,
                        ))),
                        Box::new(Const(2)),
                    ),
                    SymbolAccess(SymbolAccess::new(
                        Identifier("x".to_string()),
                        AccessType::Default,
                    )),
                )),
                None,
            )],
        )),
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("s".to_string()), 0, 0, 2),
            TraceBinding::new(Identifier("a".to_string()), 0, 2, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 3, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 4, 4),
            TraceBinding::new(Identifier("d".to_string()), 0, 8, 4),
        ]]),
        SourceSection::IntegrityConstraints(vec![ConstraintComprehension(
            ConstraintType::Evaluator(EvaluatorFunctionCall::new(
                Identifier("is_binary".to_string()),
                vec![vec![TraceBindingAccess::new(
                    Identifier("x".to_string()),
                    0,
                    TraceBindingAccessSize::Full,
                    0,
                )]],
            )),
            Some(Mul(
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("s".to_string()),
                    AccessType::Vector(0),
                ))),
                Box::new(SymbolAccess(SymbolAccess::new(
                    Identifier("s".to_string()),
                    AccessType::Vector(1),
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
