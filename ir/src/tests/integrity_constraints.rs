use super::{parse, AirIR};

#[test]
fn integrity_constraints() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn ic_using_parens() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    integrity_constraints:
        enf clk' = (clk + 1)";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn ic_op_mul() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    integrity_constraints:
        enf clk' * clk = 1";
    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn ic_op_exp() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    integrity_constraints:
        enf clk'^2 - clk = 1";
    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}
