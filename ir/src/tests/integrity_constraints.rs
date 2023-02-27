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

    let result = AirIR::new(&parsed);
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

    let result = AirIR::new(&parsed);
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

    let result = AirIR::new(&parsed);
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

    let result = AirIR::new(&parsed);
    assert!(result.is_ok());
}

#[test]
fn err_non_const_exp_outside_lc() {
    // non const exponents are not allowed outside of list comprehensions
    let source = "
    trace_columns:
        main: [clk, fmp[2], ctx]
        aux: [a, b, c[4], d[4]]
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf c[2].first = 0
    
    integrity_constraints:
        enf clk = 2^ctx";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(&parsed);
    assert!(result.is_err());
}
