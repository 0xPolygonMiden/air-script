use miden_diagnostics::{SourceSpan, Span};

use crate::ast::*;

use super::ParseTest;

// TRACE COLUMNS
// ================================================================================================

#[test]
fn trace_columns() {
    let source = r#"
    def test

    trace_columns {
        main: [clk, fmp, ctx],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf clk.first = 0;
    }

    integrity_constraints {
        enf clk = 0;
    }"#;
    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected
        .trace_columns
        .push(trace_segment!(0, "$main", [(clk, 1), (fmp, 1), (ctx, 1)]));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 2),
    );
    expected.boundary_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(eq!(
            bounded_access!(clk, Boundary::First),
            int!(0)
        ))],
    ));
    expected.integrity_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(eq!(access!(clk), int!(0)))],
    ));
    ParseTest::new().expect_module_ast(source, expected);
}

#[test]
fn trace_columns_main_and_aux() {
    let source = r#"
    def test

    trace_columns {
        main: [clk, fmp, ctx],
        aux: [rc_bus, ch_bus],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf clk.first = 0;
    }

    integrity_constraints {
        enf clk = 0;
    }"#;
    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected
        .trace_columns
        .push(trace_segment!(0, "$main", [(clk, 1), (fmp, 1), (ctx, 1)]));
    expected
        .trace_columns
        .push(trace_segment!(1, "$aux", [(rc_bus, 1), (ch_bus, 1)]));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 2),
    );
    expected.boundary_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(eq!(
            bounded_access!(clk, Boundary::First),
            int!(0)
        ))],
    ));
    expected.integrity_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(eq!(access!(clk), int!(0)))],
    ));
    ParseTest::new().expect_module_ast(source, expected);
}

#[test]
fn trace_columns_groups() {
    let source = r#"
    def test

    trace_columns {
        main: [clk, fmp, ctx, a[3]],
        aux: [rc_bus, b[4], ch_bus],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf clk.first = 0;
    }

    integrity_constraints {
        enf a[1]' = 1;
        enf clk' = clk - 1;
    }"#;
    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (fmp, 1), (ctx, 1), (a, 3)]
    ));
    expected.trace_columns.push(trace_segment!(
        1,
        "$aux",
        [(rc_bus, 1), (b, 4), (ch_bus, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 2),
    );
    expected.boundary_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(eq!(
            bounded_access!(clk, Boundary::First),
            int!(0)
        ))],
    ));
    expected.integrity_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![
            enforce!(eq!(access!(a[1], 1), int!(1))),
            enforce!(eq!(access!(clk, 1), sub!(access!(clk), int!(1)))),
        ],
    ));
    ParseTest::new().expect_module_ast(source, expected);
}

#[test]
fn err_empty_trace_columns() {
    let source = r#"
    def test

    trace_columns {}
    "#;

    // Trace columns cannot be empty
    ParseTest::new().expect_module_diagnostic(source, "trace_columns section cannot be empty");
}

#[test]
fn err_main_trace_cols_missing() {
    // returns an error if main trace columns are not defined
    let source = r#"
    def test

    trace_columns {
        aux: [clk],
    }
    public_inputs {
        stack_inputs: [16],
    }
    integrity_constraints {
        enf clk' = clk + 1;
    }
    boundary_constraints {
        enf clk.first = 0;
    }"#;

    ParseTest::new()
        .expect_module_diagnostic(source, "declaration of main trace columns is required");
}
