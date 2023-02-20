use super::{parse, AirIR};

#[test]
fn boundary_constraints() {
    let source = "
    def TestAir

    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn err_bc_duplicate_first() {
    let source = "
    def TestAir

    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
        enf clk.first = 1
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);

    assert!(result.is_err());
}

#[test]
fn err_bc_duplicate_last() {
    let source = "
    def TestAir

    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.last = 0
        enf clk.last = 1
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);

    assert!(result.is_err());
}
