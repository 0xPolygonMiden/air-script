use super::super::compile_from_source;
use super::super::expect_diagnostic;

#[test]
fn constraint_comprehension() {
    let source = "
    def test
    trace_columns {
        main: [clk, fmp[2], ctx, a, b, c[4], d[4]],
    }
    public_inputs {
        stack_inputs: [16],
    }
    boundary_constraints {
        enf c[2].first = 0;
    }
    integrity_constraints {
        enf c = d for (c, d) in (c, d);
    }";

    assert!(compile_from_source(source).is_ok());
}

#[test]
fn ic_comprehension_with_selectors() {
    let source = "
    def test
    trace_columns {
        main: [clk, fmp[2], ctx, a, b, c[4], d[4]],
    }
    public_inputs {
        stack_inputs: [16],
    }
    boundary_constraints {
        enf c[2].first = 0;
    }
    integrity_constraints {
        enf c = d for (c, d) in (c, d) when !fmp[0];
    }";

    assert!(compile_from_source(source).is_ok());
}

#[test]
fn ic_comprehension_with_tag_range() {
    let source = "
    def test
    const CURRENT_MAX_ID = 4;
    trace_columns {
        main: [clk, fmp[2], ctx, a, b, c[4], d[4]],
    }
    public_inputs {
        stack_inputs: [16],
    }
    boundary_constraints {
        @tag(0) enf c[2].first = 0;
    }
    integrity_constraints {
        @tag_range(1..5) enf c = d for (c, d) in (c, d);
    }";

    assert!(compile_from_source(source).is_ok());
}

#[test]
fn err_ic_comprehension_tag_mismatch() {
    let source = "
    def test
    const CURRENT_MAX_ID = 3;
    trace_columns {
        main: [clk, fmp[2], ctx, a, b, c[4], d[4]],
    }
    public_inputs {
        stack_inputs: [16],
    }
    boundary_constraints {
        @tag(0) enf c[2].first = 0;
    }
    integrity_constraints {
        @tag_range(1..3) enf c = d for (c, d) in (c, d);
    }";

    expect_diagnostic(source, "constraint tag count does not match comprehension length");
}
