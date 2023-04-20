use super::{parse, AirIR};

#[test]
fn simple_evaluator() {
    let source = "
    ev advance_clock([clk]):
        let z = a + 1
        enf clk' = clk + 1
    
    trace_columns:
        main: [clk]
    
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf clk.first = 0
    
    integrity_constraints:
        enf advance_clock([clk])";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn evaluator_with_variables() {
    let source = "
    ev advance_clock([clk]):
        let z = clk + 1
        enf clk' = z
    
    trace_columns:
        main: [clk]
    
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf clk.first = 0
    
    integrity_constraints:
        enf advance_clock([clk])";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn evaluator_with_main_and_aux_cols() {
    let source = "
    ev enforce_constraints([clk], [a, b]):
        let z = a + b
        enf clk' = clk + 1
        enf a' = a + z
    
    trace_columns:
        main: [clk]
        aux: [a, b]
    
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf clk.first = 0
    
    integrity_constraints:
        enf enforce_constraints([clk], [a, b])";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[ignore]
#[test]
fn ev_call_with_aux_only() {
    let source = "
    ev enforce_a([], [a, b]):
        enf a' = a + 1
    
    trace_columns:
        main: [clk]
        aux: [a, b]
    
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf clk.first = 0
    
    integrity_constraints:
        enf enforce_a([a, b])";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn ev_call_inside_evaluator_with_main() {
    let source = "
    ev enforce_clk([clk]):
        enf clk' = clk + 1
    
    ev enforce_all_constraints([clk]):
        enf enforce_clk([clk])
    
    trace_columns:
        main: [clk]
    
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf clk.first = 0
    
    integrity_constraints:
        enf enforce_all_constraints([clk])";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[ignore]
#[test]
fn ev_call_inside_evaluator_with_aux() {
    let source = "
    ev enforce_clk([clk]):
        enf clk' = clk + 1
    
    ev enforce_a([], [a, b]):
        enf a' = a + 1
    
    ev enforce_all_constraints([clk], [a, b]):
        enf enforce_clk([clk])
        enf enforce_a([], [a, b])
    
    trace_columns:
        main: [clk]
        aux: [a, b]
    
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf clk.first = 0
    
    integrity_constraints:
        enf enforce_all_constraints([clk], [a, b])";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}
