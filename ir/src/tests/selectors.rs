use super::{parse, AirIR};

#[test]
fn single_selector() {
    let source = "
    trace_columns:
        main: [s[2], clk]
    
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf clk.first = 0
    
    integrity_constraints:
        enf clk' = clk when s[0]";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn chained_selectors() {
    let source = "
    trace_columns:
        main: [s[3], clk]
    
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf clk.first = 0
    
    integrity_constraints:
        enf clk' = clk when (s[0] & !s[1]) | !s[2]'";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn multiconstraint_selectors() {
    let source = "
    trace_columns:
        main: [s[3], clk]
    
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf clk.first = 0
    
    integrity_constraints:
        enf clk' = 0 when s[0] & !s[1]
        match enf:
            clk' = clk when s[0] & s[1]
            clk' = 1 when !s[0] & !s[1]";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn selectors_in_evaluators() {
    let source = "
    ev evaluator_with_selector([selector, clk]):
        enf clk' - clk = 0 when selector
    
    trace_columns:
        main: [s[3], clk]
    
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf clk.first = 0
    
    integrity_constraints:
        enf evaluator_with_selector([s[0], clk])";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn multiple_selectors_in_evaluators() {
    let source = "
    ev evaluator_with_selector([s0, s1, clk]):
        enf clk' - clk = 0 when s0 & !s1
    
    trace_columns:
        main: [s[3], clk]
    
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf clk.first = 0
    
    integrity_constraints:
        enf evaluator_with_selector([s[0], s[1], clk])";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn selector_with_evaluator_call() {
    let source = "
    ev unchanged([clk]):
        enf clk' = clk
    
    trace_columns:
        main: [s[3], clk]
    
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf clk.first = 0
    
    integrity_constraints:
        enf unchanged([clk]) when s[0] & !s[1]";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn selectors_inside_match() {
    let source = "
    ev next_is_zero([clk]):
        enf clk' = 0

    ev is_unchanged([clk, s]):
        enf clk' = clk when s

    ev next_is_one([clk]):
        enf clk' = 1

    trace_columns:
        main: [s[3], clk]
        
    public_inputs:
        stack_inputs: [16]

    boundary_constraints:
        enf clk.first = 0

    integrity_constraints:
        enf next_is_zero([clk]) when s[0] & !s[1]
        match enf:
            is_unchanged([clk, s[0]]) when s[1] & s[2]
            next_is_one([clk]) when !s[1] & !s[2]";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}
