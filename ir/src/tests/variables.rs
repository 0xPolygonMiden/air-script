use super::{parse, AirIR};

#[test]
fn integrity_constraints_with_variables() {
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
        let a = 1
        let b = [a, a*a]
        let c = [[clk' - clk, clk - a], [1 + 8, 2^2]]
        enf c[0][0] = 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn err_bc_variable_access_before_declaration() {
    let source = "
    const A = [[2, 3], [1, 0]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = a
        let a = 0
        enf clk.last = 1
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_ic_variable_access_before_declaration() {
    let source = "
    const A = [[2, 3], [1, 0]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1
    integrity_constraints:
        enf clk' = clk + a
        let a = 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_variable_vector_invalid_access() {
    let source = "
    const A = [[2, 3], [1, 0]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1
    integrity_constraints:
        let a = [1, 2]
        enf clk' = clk + a[2]";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_variable_matrix_invalid_access() {
    let source = "
    const A = [[2, 3], [1, 0]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1
    integrity_constraints:
        let a = [[1, 2, 3], [4, 5, 6]]
        enf clk' = clk + a[1][3]";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());

    let source = "
    const A = [[2, 3], [1, 0]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1
    integrity_constraints:
        let a = [[1, 2, 3], [4, 5, 6]]
        enf clk' = clk + a[2][0]";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}
