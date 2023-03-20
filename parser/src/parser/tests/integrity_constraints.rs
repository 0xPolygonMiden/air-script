use super::{build_parse_test, Identifier, IntegrityConstraint, Source, SourceSection};
use crate::{
    ast::{
        Constant, ConstantType::*, ConstraintType, Expression::*, IndexedTraceAccess,
        IntegrityStmt::*, MatrixAccess, NamedTraceAccess, Range, Slice, Variable, VariableType,
        VectorAccess,
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
            NamedTraceAccess(NamedTraceAccess::new(Identifier("clk".to_string()), 0, 1)),
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
                NamedTraceAccess(NamedTraceAccess::new(Identifier("clk".to_string()), 0, 1)),
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
                    Box::new(NamedTraceAccess(NamedTraceAccess::new(
                        Identifier("clk".to_string()),
                        0,
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
fn integrity_constraint_with_periodic_col() {
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
fn integrity_constraint_with_random_value() {
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
fn integrity_constraint_with_slice() {
    let source = "
    integrity_constraints:
        let c = d[1..3]
        enf a + c[1] = 0";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![
        Variable(Variable::new(
            Identifier("c".to_string()),
            VariableType::Scalar(Slice(Slice::new(
                Identifier("d".to_string()),
                Range::new(1, 3),
            ))),
        )),
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                Add(
                    Box::new(Elem(Identifier("a".to_string()))),
                    Box::new(VectorAccess(VectorAccess::new(
                        Identifier("c".to_string()),
                        1,
                    ))),
                ),
                Const(0),
            )),
            None,
        ),
    ])]);
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
fn integrity_constraint_with_variables() {
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
fn integrity_constraint_with_indexed_trace_access() {
    let source = "
    integrity_constraints:
        enf $main[0]' = $main[1] + 1
        enf $aux[0]' - $aux[1] = 1";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                IndexedTraceAccess(IndexedTraceAccess::new(0, 0, 1)),
                Add(
                    Box::new(IndexedTraceAccess(IndexedTraceAccess::new(0, 1, 0))),
                    Box::new(Const(1)),
                ),
            )),
            None,
        ),
        Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                Sub(
                    Box::new(IndexedTraceAccess(IndexedTraceAccess::new(1, 0, 1))),
                    Box::new(IndexedTraceAccess(IndexedTraceAccess::new(1, 1, 0))),
                ),
                Const(1),
            )),
            None,
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}

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

// UNRECOGNIZED TOKEN ERRORS
// ================================================================================================

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
