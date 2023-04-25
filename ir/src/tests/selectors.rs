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
