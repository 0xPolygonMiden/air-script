use super::expect_diagnostic;

#[test]
fn err_missing_current_max_id() {
    let source = "
    def test
    trace_columns {
        main: [clk, a],
    }
    public_inputs {
        stack_inputs: [16],
    }
    boundary_constraints {
        @tag(0) enf a.first = 0;
    }
    integrity_constraints {
        @tag(1) enf clk = a;
    }";

    expect_diagnostic(source, "missing CURRENT_MAX_ID constant for tagged constraints");
}

#[test]
fn err_duplicate_tag() {
    let source = "
    def test
    const CURRENT_MAX_ID = 1;
    trace_columns {
        main: [clk, a],
    }
    public_inputs {
        stack_inputs: [16],
    }
    boundary_constraints {
        @tag(0) enf a.first = 0;
    }
    integrity_constraints {
        @tag(0) enf clk = a;
    }";

    expect_diagnostic(source, "duplicate constraint tag");
}

#[test]
fn err_tag_gap() {
    let source = "
    def test
    const CURRENT_MAX_ID = 2;
    trace_columns {
        main: [clk, a],
    }
    public_inputs {
        stack_inputs: [16],
    }
    boundary_constraints {
        @tag(0) enf a.first = 0;
    }
    integrity_constraints {
        @tag(2) enf clk = a;
    }";

    expect_diagnostic(source, "constraint tag count does not match CURRENT_MAX_ID");
}

#[test]
fn err_tag_out_of_range() {
    let source = "
    def test
    const CURRENT_MAX_ID = 1;
    trace_columns {
        main: [clk, a],
    }
    public_inputs {
        stack_inputs: [16],
    }
    boundary_constraints {
        @tag(2) enf a.first = 0;
    }
    integrity_constraints {
        @tag(0) enf clk = a;
    }";

    expect_diagnostic(source, "constraint tag exceeds CURRENT_MAX_ID");
}

#[test]
fn err_missing_current_max_id_in_evaluator() {
    let source = "
    def test
    ev advance_clock([clk]) {
        @tag(0) enf clk' = clk + 1;
    }
    trace_columns {
        main: [clk],
    }
    public_inputs {
        stack_inputs: [16],
    }
    boundary_constraints {
        enf clk.first = 0;
    }
    integrity_constraints {
        enf advance_clock([clk]);
    }";

    expect_diagnostic(source, "missing CURRENT_MAX_ID constant for tagged constraints");
}
