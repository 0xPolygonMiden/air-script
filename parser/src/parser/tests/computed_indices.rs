use miden_diagnostics::{SourceSpan, Span};

use super::ParseTest;
use crate::ast::*;

#[test]
fn basic_computed_indices() {
    let source = "
    def test

    trace_columns {
        main: [a, b, c[4]],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf a.first = 0;
    }

    integrity_constraints {
        let x = [0, 1, 2, 3, 4];

        enf a = x[1 + 1];
    }";

    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected.trace_columns.push(trace_segment!(
        TraceSegmentId::Main,
        "$main",
        [(a, 1), (b, 1), (c, 4)]
    ));
    expected
        .public_inputs
        .insert(ident!(inputs), PublicInput::new_vector(SourceSpan::UNKNOWN, ident!(inputs), 2));
    expected.boundary_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(eq!(bounded_access!(a, Boundary::First), int!(0)))],
    ));
    expected.integrity_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![let_!(x = vector!(int!(0), int!(1), int!(2), int!(3), int!(4)) =>
                enforce!(eq!(access!(a), access!(x[Box::new(add!(int!(1), int!(1)))]))))],
    ));

    ParseTest::new().expect_module_ast(source, expected);
}

#[test]
fn basic_computed_indices_in_lc() {
    let source = "
    def test

    trace_columns {
        main: [a, b, c[4]],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf a.first = 0;
    }

    integrity_constraints {
        let x = [0, 1, 2, 3, 4];
        let y = [i * x[1 + 1] for i in 0..5];

        enf a = y[1 + 1];
    }";

    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected.trace_columns.push(trace_segment!(
        TraceSegmentId::Main,
        "$main",
        [(a, 1), (b, 1), (c, 4)]
    ));
    expected
        .public_inputs
        .insert(ident!(inputs), PublicInput::new_vector(SourceSpan::UNKNOWN, ident!(inputs), 2));
    expected.boundary_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(eq!(bounded_access!(a, Boundary::First), int!(0)))],
    ));
    expected.integrity_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![let_!(x = vector!(int!(0), int!(1), int!(2), int!(3), int!(4)) =>
                let_!(y = lc!(((i, range!(0usize, 5usize))) => mul!(access!(i), access!(x[Box::new(add!(int!(1), int!(1)))]))).into() =>
                    enforce!(eq!(access!(a), access!(y[Box::new(add!(int!(1), int!(1)))])))))],
    ));

    ParseTest::new().expect_module_ast(source, expected);
}

#[test]
fn computed_indices_in_lc() {
    let source = "
    def test

    trace_columns {
        main: [a, b, c[4]],
    }

    public_inputs {
        inputs: [2],
    }

    boundary_constraints {
        enf a.first = 0;
    }

    integrity_constraints {
        let x = [0, 1, 2, 3, 4];
        let y = [i * x[i + 1] for i in 0..4];

        enf a = y[1 + 1];
    }";

    let mut expected = Module::new(ModuleType::Root, SourceSpan::UNKNOWN, ident!(test));
    expected.trace_columns.push(trace_segment!(
        TraceSegmentId::Main,
        "$main",
        [(a, 1), (b, 1), (c, 4)]
    ));
    expected
        .public_inputs
        .insert(ident!(inputs), PublicInput::new_vector(SourceSpan::UNKNOWN, ident!(inputs), 2));
    expected.boundary_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![enforce!(eq!(bounded_access!(a, Boundary::First), int!(0)))],
    ));
    expected.integrity_constraints = Some(Span::new(
        SourceSpan::UNKNOWN,
        vec![let_!(x = vector!(int!(0), int!(1), int!(2), int!(3), int!(4)) =>
                let_!(y = lc!(((i, range!(0usize, 4usize))) => mul!(access!(i), access!(x[Box::new(add!(access!(i), int!(1)))]))).into() =>
                    enforce!(eq!(access!(a), access!(y[Box::new(add!(int!(1), int!(1)))])))))],
    ));

    ParseTest::new().expect_module_ast(source, expected);
}
