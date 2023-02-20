use super::{parse, AirIR};

#[test]
fn bc_with_public_inputs() {
    let source = "
    def TestAir
    
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = stack_inputs[0]^3
    integrity_constraints:
        enf clk' = clk - 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}
