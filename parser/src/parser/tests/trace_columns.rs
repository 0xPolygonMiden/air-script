use crate::ast::TraceColAccess;

use super::{
    build_parse_test, Error, Identifier, ParseError, Source, SourceSection::*, TraceCol, TraceCols,
    TransitionConstraint, TransitionExpr::*, TransitionStmt::*,
};

// TRACE COLUMNS
// ================================================================================================

#[test]
fn trace_columns() {
    let source = "
    trace_columns:
        main: [clk, fmp, ctx]";
    let expected = Source(vec![TraceCols(TraceCols {
        main_cols: vec![
            TraceCol::Single(Identifier("clk".to_string())),
            TraceCol::Single(Identifier("fmp".to_string())),
            TraceCol::Single(Identifier("ctx".to_string())),
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
    let expected = Source(vec![TraceCols(TraceCols {
        main_cols: vec![
            TraceCol::Single(Identifier("clk".to_string())),
            TraceCol::Single(Identifier("fmp".to_string())),
            TraceCol::Single(Identifier("ctx".to_string())),
        ],
        aux_cols: vec![
            TraceCol::Single(Identifier("rc_bus".to_string())),
            TraceCol::Single(Identifier("ch_bus".to_string())),
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
    transition_constraints:
        enf a[1]' = 1";
    let expected = Source(vec![
        TraceCols(TraceCols {
            main_cols: vec![
                TraceCol::Single(Identifier("clk".to_string())),
                TraceCol::Single(Identifier("fmp".to_string())),
                TraceCol::Single(Identifier("ctx".to_string())),
                TraceCol::Group(Identifier("a".to_string()), 3),
            ],
            aux_cols: vec![
                TraceCol::Single(Identifier("rc_bus".to_string())),
                TraceCol::Group(Identifier("b".to_string()), 4),
                TraceCol::Single(Identifier("ch_bus".to_string())),
            ],
        }),
        TransitionConstraints(vec![Constraint(TransitionConstraint::new(
            Next(TraceColAccess::GroupAccess(Identifier("a".to_string()), 1)),
            Const(1),
        ))]),
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
    transition_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = 0";

    let error = Error::ParseError(ParseError::MissingMainTraceCols(
        "Declaration of main trace columns is required".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}
