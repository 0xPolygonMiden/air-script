use super::{parse, AirIR};

#[test]
fn constraint_comprehension() {
    let source = "
    trace_columns:
        main: [clk, fmp[2], ctx]
        aux: [a, b, c[4], d[4]]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf c[2].first = 0
    integrity_constraints:
        enf c = d for (c, d) in (c, d)";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn ic_comprehension_with_selectors() {
    let source = "
    trace_columns:
        main: [clk, fmp[2], ctx]
        aux: [a, b, c[4], d[4]]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf c[2].first = 0
    integrity_constraints:
        enf c = d for (c, d) in (c, d) when !fmp[0]";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}
