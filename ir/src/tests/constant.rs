use super::{parse, AirIR};

#[test]
fn bc_with_constants() {
    let source = "
    const A = 123
    const B = [1, 2, 3]
    const C = [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = A
        enf clk.last = B[0] + C[0][1]
    integrity_constraints:
        enf clk' = clk - 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    println!("{result:?}");
    assert!(result.is_ok());
}

#[test]
fn ic_with_constants() {
    let source = "
    const A = 123
    const B = [1, 2, 3]
    const C = [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    integrity_constraints:
        enf clk' = clk + A + B[1] - C[1][2]";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(parsed);
    println!("{result:?}");
    assert!(result.is_ok());
}

#[test]
fn err_invalid_matrix_const() {
    let source = "
    const A = [[2, 3], [1, 0, 2]]
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
    let result = AirIR::new(parsed);
    assert!(result.is_err());
}
