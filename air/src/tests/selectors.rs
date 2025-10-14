use super::compile_from_source;

#[test]
fn single_selector() {
    let source = "
    def test
    trace_columns {
        main: [s[2], clk],
    }

    public_inputs {
        stack_inputs: [16],
    }
    boundary_constraints {
        enf clk.first = 0;
    }
    integrity_constraints {
        enf clk' = clk when s[0];
    }";

    assert!(compile_from_source(source).is_ok());
}

#[test]
fn chained_selectors() {
    let source = "
    def test
    trace_columns {
        main: [s[3], clk],
    }
    
    public_inputs {
        stack_inputs: [16],
    }
    boundary_constraints {
        enf clk.first = 0;
    }
    integrity_constraints {
        enf clk' = clk when (s[0] & !s[1]) | !s[2]';
    }";

    assert!(compile_from_source(source).is_ok());
}

#[test]
fn multiconstraint_selectors() {
    let source = "
    def test
    trace_columns {
        main: [s[3], clk],
    }

    public_inputs {
        stack_inputs: [16],
    }

    boundary_constraints {
        enf clk.first = 0;
    }

    integrity_constraints {
        enf clk' = 0 when s[0] & !s[1];
        enf match {
            case s[0] & s[1]: clk' = clk,
            case !s[0] & !s[1]: clk' = 1,
        };
    }";

    assert!(compile_from_source(source).is_ok());
}

#[test]
fn selectors_in_evaluators() {
    let source = "
    def test
    ev evaluator_with_selector([selector, clk]) {
        enf clk' - clk = 0 when selector;
    }
    
    trace_columns {
        main: [s[3], clk],
    }

    public_inputs {
        stack_inputs: [16],
    }

    boundary_constraints {
        enf clk.first = 0;
    }

    integrity_constraints {
        enf evaluator_with_selector([s[0], clk]);
    }";

    assert!(compile_from_source(source).is_ok());
}

#[test]
fn multiple_selectors_in_evaluators() {
    let source = "
    def test
    ev evaluator_with_selector([s0, s1, clk]) {
        enf clk' - clk = 0 when s0 & !s1;
    }
    
    trace_columns {
        main: [s[3], clk],
    }

    public_inputs {
        stack_inputs: [16],
    }

    boundary_constraints {
        enf clk.first = 0;
    }

    integrity_constraints {
        enf evaluator_with_selector([s[0], s[1], clk]);
    }";

    assert!(compile_from_source(source).is_ok());
}

#[test]
fn selector_with_evaluator_call() {
    let source = "
    def test
    ev unchanged([clk]) {
        enf clk' = clk;
    }
    
    trace_columns {
        main: [s[3], clk],
    }

    public_inputs {
        stack_inputs: [16],
    }

    boundary_constraints {
        enf clk.first = 0;
    }

    integrity_constraints {
        enf unchanged([clk]) when s[0] & !s[1];
    }";

    assert!(compile_from_source(source).is_ok());
}

#[test]
fn selectors_inside_match() {
    let source = "
    def test
    ev next_is_zero([clk]) {
        enf clk' = 0;
    }

    ev is_unchanged([clk, s]) {
        enf clk' = clk when s;
    }

    ev next_is_one([clk]) {
        enf clk' = 1;
    }

    trace_columns {
        main: [s[3], clk],
    }
 
    public_inputs {
        stack_inputs: [16],
    }

    boundary_constraints {
        enf clk.first = 0;
    }

    integrity_constraints {
        enf next_is_zero([clk]) when s[0] & !s[1];
        enf match {
            case s[1] & s[2]: is_unchanged([clk, s[0]]),
            case !s[1] & !s[2]: next_is_one([clk]),
        };
    }";

    assert!(compile_from_source(source).is_ok());
}

/// This test ensures that nested selectors are well handled during compilation by the
/// MatchOptimizer pass.
#[test]
fn selectors_nested() {
    let source = "
    def test

    const R = [8, 4, 20, 31, 5, 15];

    trace_columns {
        main: [s[3], a, b, c],
    }

    public_inputs {
        stack_inputs: [1],
    }

    boundary_constraints {
        enf c.first = 0;
    }

    # Simple evaluator functions
    ev ev_dummy_0([b]) {
        enf b' = b + 1;
    }

    ev ev_dummy_1([b]) {
        enf b' = b;
    }

    ev ev_s0([a, b, c]) {
        enf c = R[2];
    }  

    ev ev_s2([a, b, c]) {
        enf b = R[4];
    }

    # Evaluator with nested match statement
    ev ev_s1([a, b, c]) {
        enf a * (a - 1) = 0;
        enf b = 31;
        enf c = 5;

        # This creates Vector nodes with Enf operations
        enf match {
            case a: ev_dummy_0([b]),
            case !a: ev_dummy_1([b]),
        };
    }

    # Main constraint with nested evaluator calls
    integrity_constraints {
        let s0 = s[0];
        let s1 = !s[0] & s[1];
        let s2 = !s[0] & !s[1];

        # This pattern creates the problematic Vector structures
        enf match {
            case s0: ev_s0([a, b, c]),
            case s1: ev_s1([a, b, c]),    # ← This call creates nested Vector->Enf structures
            case s2: ev_s2([a, b, c]),
        };
    }";

    assert!(compile_from_source(source).is_ok());
}
