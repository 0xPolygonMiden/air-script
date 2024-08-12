use miden_diagnostics::{SourceSpan, Span};

use crate::ast::*;

use super::ParseTest;

// RANDOM VALUES
// ================================================================================================

#[test]
fn random_values_fixed_list() {
    let source = "
    def test

    trace_columns {
        main: [clk],
        aux: [a],
    }

    random_values {
        rand: [15],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf clk.first = 0;
    }

    integrity_constraints {
        enf clk = 0;
    }";

    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected
        .trace_columns
        .push(trace_segment!(0, "$main", [(clk, 1)]));
    expected
        .trace_columns
        .push(trace_segment!(1, "$aux", [(a, 1)]));
    expected.random_values = Some(random_values!("$rand", 15));
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
fn random_values_ident_vector() {
    let source = "
    def test

    trace_columns {
        main: [clk],
        aux: [aux0],
    }

    random_values {
        rand: [a, b[12], c],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf clk.first = 0;
    }

    integrity_constraints {
        enf clk = 0;
    }";

    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected
        .trace_columns
        .push(trace_segment!(0, "$main", [(clk, 1)]));
    expected
        .trace_columns
        .push(trace_segment!(1, "$aux", [(aux0, 1)]));
    expected.random_values = Some(random_values!("$rand", [(a, 1), (b, 12), (c, 1)]));
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
fn random_values_custom_name() {
    let source = "
    def test

    trace_columns {
        main: [clk],
        aux: [aux0],
    }

    random_values {
        alphas: [14],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf clk.first = 0;
    }

    integrity_constraints {
        enf clk = 0;
    }";

    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected
        .trace_columns
        .push(trace_segment!(0, "$main", [(clk, 1)]));
    expected
        .trace_columns
        .push(trace_segment!(1, "$aux", [(aux0, 1)]));
    expected.random_values = Some(random_values!("$alphas", 14));
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
fn err_random_values_empty_list() {
    let source = "
    def test

    trace_columns {
        main: [clk],
    }

    random_values {
        rand: [],
    }

    integrity_constraints {
        enf clk = 0;
    }";

    ParseTest::new().expect_module_diagnostic(source, "random values cannot be empty");
}

#[test]
fn err_random_values_multiple_declaration() {
    let source = "
    def test

    trace_columns {
        main: [clk],
    }

    random_values {
        rand: [12],
        alphas: [a, b[2]],
    }

    integrity_constraints {
        enf clk = 0;
    }";

    ParseTest::new()
        .expect_module_diagnostic(source, "only one declaration may appear in random_values");
}

#[test]
fn random_values_index_access() {
    let source = "
    def test

    trace_columns {
        main: [clk],
        aux: [aux0],
    }

    random_values {
        rand: [12],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf clk.first = 0;
    }

    integrity_constraints {
        enf clk + $rand[1] = 0;
    }";

    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected
        .trace_columns
        .push(trace_segment!(0, "$main", [(clk, 1)]));
    expected
        .trace_columns
        .push(trace_segment!(1, "$aux", [(aux0, 1)]));
    expected.random_values = Some(random_values!("$rand", 12));
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
        vec![enforce!(eq!(
            add!(access!(clk), access!("$rand"[1])),
            int!(0)
        ))],
    ));
    ParseTest::new().expect_module_ast(source, expected);
}
