use super::{build_parse_test, Error, Identifier, ParseError, Source, SourceSection::*, TraceCols};

// TRACE COLUMNS
// ================================================================================================

#[test]
fn trace_columns() {
    let source = "
    trace_columns:
        main: [clk, fmp, ctx]";
    let expected = Source(vec![TraceCols(TraceCols {
        main_cols: vec![
            Identifier("clk".to_string()),
            Identifier("fmp".to_string()),
            Identifier("ctx".to_string()),
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
            Identifier("clk".to_string()),
            Identifier("fmp".to_string()),
            Identifier("ctx".to_string()),
        ],
        aux_cols: vec![
            Identifier("rc_bus".to_string()),
            Identifier("ch_bus".to_string()),
        ],
    })]);
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
