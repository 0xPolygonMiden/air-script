use super::{parse, AirIR};

#[test]
fn list_folding_on_const() {
    let source = "
    const A = [1, 2, 3]
    trace_columns:
        main: [clk, fmp[2], ctx]
        aux: [a, b, c[4], d[4]]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf c[2].first = 0
    integrity_constraints:
        let x = sum(A)
        let y = prod(A)
        enf clk = y - x";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(&parsed);
    assert!(result.is_ok());
}

#[test]
fn list_folding_on_variable() {
    let source = "
    trace_columns:
        main: [clk, fmp[2], ctx]
        aux: [a, b, c[4], d[4]]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf c[2].first = 0
    integrity_constraints:
        let x = [a + c[0], 1, c[2] * d[2]]
        let y = sum(x)
        let z = prod(x)
        enf clk = z - y";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(&parsed);
    assert!(result.is_ok());
}

#[test]
fn list_folding_on_vector() {
    let source = "
    trace_columns:
        main: [clk, fmp[2], ctx]
        aux: [a, b, c[4], d[4]]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf c[2].first = 0
    integrity_constraints:
        let x = sum([c[0], c[2], 2 * a])
        let y = prod([c[0], c[2], 2 * a])
        enf clk = y - x";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(&parsed);
    assert!(result.is_ok());
}

#[test]
fn list_folding_on_lc() {
    let source = "
    const A = [1, 2, 3]
    trace_columns:
        main: [clk, fmp[2], ctx]
        aux: [a, b, c[4], d[4]]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf c[2].first = 0
    integrity_constraints:
        let x = sum([c * d for (c, d) in (c, d)])
        let y = prod([c + d for (c, d) in (c, d)])    
        enf clk = y - x";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(&parsed);
    assert!(result.is_ok());
}

#[test]
fn list_folding_in_lc() {
    let source = "
    trace_columns:
        main: [clk, fmp[4], ctx]
        aux: [a, b, c[4], d[4]]
    public_inputs:
        stack_inputs: [16]
    
    boundary_constraints:
        enf c[2].first = 0
    
    integrity_constraints:
        let x = sum([c * d for (c, d) in (c, d)])
        let y = [m + x for m in fmp]
        enf clk = y[0]";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(&parsed);

    assert!(result.is_ok());
}
