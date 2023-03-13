use crate::ast::ConstraintType;

use super::{
    build_parse_test, Error, Expression::*, Identifier, IntegrityConstraint, IntegrityStmt::*,
    NamedTraceAccess, ParseError, Source, SourceSection::*, Trace, TraceCols,
};

// TRACE COLUMNS
// ================================================================================================

#[test]
fn trace_columns() {
    let source = "
    trace_columns:
        main: [clk, fmp, ctx]";
    let expected = Source(vec![Trace(Trace {
        main_cols: vec![
            TraceCols::new(Identifier("clk".to_string()), 1),
            TraceCols::new(Identifier("fmp".to_string()), 1),
            TraceCols::new(Identifier("ctx".to_string()), 1),
        ],
        aux_cols: vec![],
    })]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn trace_columns_main_and_aux() {
    let source = "
    trace_columns:
        main: [clk, fmp, ctx]
        aux: [rc_bus, ch_bus]";
    let expected = Source(vec![Trace(Trace {
        main_cols: vec![
            TraceCols::new(Identifier("clk".to_string()), 1),
            TraceCols::new(Identifier("fmp".to_string()), 1),
            TraceCols::new(Identifier("ctx".to_string()), 1),
        ],
        aux_cols: vec![
            TraceCols::new(Identifier("rc_bus".to_string()), 1),
            TraceCols::new(Identifier("ch_bus".to_string()), 1),
        ],
    })]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn trace_columns_groups() {
    let source = "
    trace_columns:
        main: [clk, fmp, ctx, a[3]]
        aux: [rc_bus, b[4], ch_bus]
    integrity_constraints:
        enf a[1]' = 1
        enf clk' = clk - 1";
    let expected = Source(vec![
        Trace(Trace {
            main_cols: vec![
                TraceCols::new(Identifier("clk".to_string()), 1),
                TraceCols::new(Identifier("fmp".to_string()), 1),
                TraceCols::new(Identifier("ctx".to_string()), 1),
                TraceCols::new(Identifier("a".to_string()), 3),
            ],
            aux_cols: vec![
                TraceCols::new(Identifier("rc_bus".to_string()), 1),
                TraceCols::new(Identifier("b".to_string()), 4),
                TraceCols::new(Identifier("ch_bus".to_string()), 1),
            ],
        }),
        IntegrityConstraints(vec![
            Constraint(ConstraintType::IntegrityConstraint(IntegrityConstraint::new(
                NamedTraceAccess(NamedTraceAccess::new(Identifier("a".to_string()), 1, 1)),
                Const(1),
            ))),
            Constraint(ConstraintType::IntegrityConstraint(IntegrityConstraint::new(
                NamedTraceAccess(NamedTraceAccess::new(Identifier("clk".to_string()), 0, 1)),
                Sub(
                    Box::new(Elem(Identifier("clk".to_string()))),
                    Box::new(Const(1)),
                ),
            ))),
        ]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn empty_trace_columns_error() {
    let source = "
    trace_columns:";
    // Trace columns cannot be empty
    let error = Error::ParseError(ParseError::InvalidTraceCols(
        "Trace Columns cannot be empty".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

#[test]
fn main_trace_cols_missing_error() {
    // returns an error if main trace columns are not defined
    let source = "
    trace_columns:
        aux: [clk]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = 0";

    let error = Error::ParseError(ParseError::MissingMainTraceCols(
        "Declaration of main trace columns is required".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}
