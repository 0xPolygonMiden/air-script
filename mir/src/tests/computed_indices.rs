use super::{compile, expect_diagnostic};

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

    assert!(compile(source).is_ok());
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

    assert!(compile(source).is_ok());
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

    assert!(compile(source).is_ok());
}

// Tests that should return errors
#[test]
fn err_computed_indices_in_lc() {
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
        let y = [i * x[a + 1] for i in 0..4];

        enf a = y[1 + 1];
    }";

    expect_diagnostic(source, "error: the index is not constant during constant propagation");
}
