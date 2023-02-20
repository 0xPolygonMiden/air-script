use super::{parse, AirIR};

#[test]
fn err_trace_cols_empty_or_omitted() {
    // if trace columns is empty, an error should be returned at parser level.
    let source = "
    def TestAir
    
    trace_columns:
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    integrity_constraints:
        enf clk' = clk + 1";

    assert!(parse(source).is_err());

    // returns an error if trace columns declaration is missing
    let source = "
    def TestAir

    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);

    // this fails before the check for missing trace columns declaration since the clk column
    // used in constraints is not declared.
    assert!(result.is_err());
}

#[test]
fn err_pub_inputs_empty_or_omitted() {
    // if public inputs are empty, an error should be returned at parser level.
    let source = "
    def TestAir

    trace_columns:
        main: [clk]
    public_inputs:
    boundary_constraints:
        enf clk.first = 0
    integrity_constraints:
        enf clk' = clk + 1";

    assert!(parse(source).is_err());

    // if public inputs are omitted, an error should be returned at IR level.
    let source = "
    def TestAir

    trace_columns:
        main: [clk]
    boundary_constraints:
        enf clk.first = 0
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_bc_empty_or_omitted() {
    // if boundary constraints are empty, an error should be returned at parser level.
    let source = "
    def TestAir

    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
    integrity_constraints:
        enf clk' = clk + 1";

    assert!(parse(source).is_err());

    // if boundary constraints are omitted, an error should be returned at IR level.
    let source = "
    def TestAir

    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_ic_empty_or_omitted() {
    // if integrity constraints are empty, an error should be returned at parser level.
    let source = "
    def TestAir

    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    integrity_constraints:";

    assert!(parse(source).is_err());

    // if integrity constraints are omitted, an error should be returned at IR level.
    let source = "
    def TestAir

    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}
