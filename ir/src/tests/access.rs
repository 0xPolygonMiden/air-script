use super::{parse, AirIR};

#[test]
fn err_bc_invalid_vector_access() {
    let source = "
    const A = 123
    const B = [1, 2, 3]
    const C = [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = A + B[3] - C[1][2]
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_bc_invalid_matrix_access() {
    let source = "
    const A = 123
    const B = [1, 2, 3]
    const C = [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = A + B[1] - C[3][2]
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(&parsed);
    assert!(result.is_err());

    let source = "
    const A = 123
    const B = [1, 2, 3]
    const C = [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = A + B[1] - C[1][3]
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_ic_invalid_vector_access() {
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
        enf clk' = clk + A + B[3] - C[1][2]";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_ic_invalid_matrix_access() {
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
        enf clk' = clk + A + B[1] - C[3][2]";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(&parsed);
    assert!(result.is_err());

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
        enf clk' = clk + A + B[1] - C[1][3]";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(&parsed);
    assert!(result.is_err());
}
