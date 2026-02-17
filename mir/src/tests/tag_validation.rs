use super::{compile, expect_diagnostic};

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
fn ok_duplicate_tags_with_current_max_id_in_mir() {
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

    assert!(compile(source).is_ok());
}

#[test]
fn ok_tag_gap_with_current_max_id_in_mir() {
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

    assert!(compile(source).is_ok());
}
