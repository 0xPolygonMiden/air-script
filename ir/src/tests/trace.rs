use super::{parse, AirIR};

#[test]
fn trace_columns_index_access() {
    let source = "
    trace_columns:
        main: [a, b]
        aux: [c, d]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf a.first = 1
    integrity_constraints:
        enf $main[0]' - $main[1] = 0
        enf $aux[0]^3 - $aux[1]' = 0";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(&parsed);
    assert!(result.is_ok());
}

#[test]
fn trace_cols_groups() {
    let source = "
    const A = 123
    const B = [1, 2, 3]
    const C = [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk, a[4]]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf a[1].first = A
        enf clk.last = B[0] + C[0][1]
    integrity_constraints:
        enf a[0]' = a[1] - 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(&parsed);
    assert!(result.is_ok());
}

#[test]
fn err_bc_column_undeclared() {
    let source = "
    trace_columns:
        main: [ctx]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_ic_column_undeclared() {
    let source = "
    trace_columns:
        main: [ctx]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf ctx.first = 0
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_bc_trace_cols_access_out_of_bounds() {
    // out of bounds in boundary constraints
    let source = "
    const A = 123
    const B = [1, 2, 3]
    const C = [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk, a[4]]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf a[4].first = A
    integrity_constraints:
        enf a[0]' = a[0] - 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_ic_trace_cols_access_out_of_bounds() {
    // out of bounds in integrity constraints
    let source = "
    const A = 123
    const B = [1, 2, 3]
    const C = [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk, a[4]]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf a[1].first = A
        enf clk.last = B[0] + C[0][1]
    integrity_constraints:
        enf a[4]' = a[4] - 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_ic_trace_cols_group_used_as_scalar() {
    let source = "
    trace_columns:
        main: [clk, a[4]]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf a[1].first = 0
    integrity_constraints:
        enf a[0]' = a + clk";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(&parsed);
    assert!(result.is_err());
}
