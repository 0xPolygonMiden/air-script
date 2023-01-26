use crate::AirIR;
use parser::parse;

#[test]
fn boundary_constraints() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn test_test() {
    let source = "
    trace_columns:
        main: [a, b[12]]
    public_inputs:
        stack_inputs: [16]
    random_values:
        rand: [c, d]
    integrity_constraints:
        enf a' = a + 1
    boundary_constraints:
        enf a.first = (c + stack_inputs[0]) * 2
        enf a.last = 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed).unwrap();
    println!("{:?}", result.boundary_stmts);
    // assert!(result.is_ok());
}

#[test]
fn boundary_constraints_with_constants() {
    let source = "
    const A = 123
    const B = [1, 2, 3]
    const C = [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        enf clk' = clk - 1
    boundary_constraints:
        enf clk.first = A
        enf clk.last = B[0] + C[0][1]";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn trace_columns_index_access() {
    let source = "
    trace_columns:
        main: [a, b]
        aux: [c, d]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        enf $main[0]' - $main[1] = 0
        enf $aux[0]^3 - $aux[1]' = 0
    boundary_constraints:
        enf a.first = stack_inputs[0]^3";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
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
    integrity_constraints:
        enf a[0]' = a[1] - 1
    boundary_constraints:
        enf a[1].first = A
        enf clk.last = B[0] + C[0][1]";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn err_trace_cols_access_out_of_bounds() {
    // out of bounds in integrity constraints
    let source = "
    const A = 123
    const B = [1, 2, 3]
    const C = [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk, a[4]]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        enf a[4]' = a[4] - 1
    boundary_constraints:
        enf a[1].first = A
        enf clk.last = B[0] + C[0][1]";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());

    // out of bounds in boundary constraints
    let source = "
    const A = 123
    const B = [1, 2, 3]
    const C = [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk, a[4]]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        enf a[0]' = a[0] - 1
    boundary_constraints:
        enf a[4].first = A
        enf clk.last = B[0] + C[0][1]";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_tc_invalid_vector_access() {
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

    let result = AirIR::from_source(&parsed);
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

    let result = AirIR::from_source(&parsed);
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

    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
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

    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_bc_empty_or_omitted() {
    // if boundary constraints are empty, an error should be returned at parser level.
    let source = "
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
fn err_bc_duplicate_first() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = 0
        enf clk.first = 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);

    assert!(result.is_err());
}

#[test]
fn err_bc_duplicate_last() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.last = 0
        enf clk.last = 1";

    let parsed = parse(source).expect("Parsing failed");

    assert!(AirIR::from_source(&parsed).is_err());
}

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

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn integrity_constraints_with_constants() {
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

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

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
    integrity_constraints:
        let a = 1
        let b = [a, a*a]
        let c = [[clk' - clk, clk - a], [1 + 8, 2^2]]
        enf c[0][0] = 1
    boundary_constraints:
        enf clk.first = A
        enf clk.last = B[0] + C[0][1]";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn transition_constraints_using_parens() {
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

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

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

    let result = AirIR::from_source(&parsed);
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

    let result = AirIR::from_source(&parsed);
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

    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_ic_empty_or_omitted() {
    // if integrity constraints are empty, an error should be returned at parser level.
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
    boundary_constraints:
        enf clk.first = 0";

    assert!(parse(source).is_err());

    // if integrity constraints are omitted, an error should be returned at IR level.
    let source = "
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

    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_public_inputs_empty_or_omitted() {
    // if public inputs are empty, an error should be returned at parser level.
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
    integrity_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = 0";

    assert!(parse(source).is_err());

    // if public inputs are omitted, an error should be returned at IR level.
    let source = "
    trace_columns:
        main: [clk]
    integrity_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = 0";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_trace_cols_omitted() {
    // returns an error if trace columns declaration is missing
    let source = "
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = 0";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);

    // this fails before the check for missing trace columns declaration since the clk column
    // used in constraints is not declared.
    assert!(result.is_err());
}

#[test]
fn op_mul() {
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

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn op_exp() {
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

    let result = AirIR::from_source(&parsed);
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
    integrity_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_bc_variable_access_before_declaration() {
    let source = "
    const A = [[2, 3], [1, 0]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = a
        let a = 0
        enf clk.last = 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_tc_variable_access_before_declaration() {
    let source = "
    const A = [[2, 3], [1, 0]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        enf clk' = clk + a
        let a = 1
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1";

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
    integrity_constraints:
        let a = [1, 2]
        enf clk' = clk + a[2]
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1";

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
    integrity_constraints:
        let a = [[1, 2, 3], [4, 5, 6]]
        enf clk' = clk + a[1][3]
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());

    let source = "
    const A = [[2, 3], [1, 0]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        let a = [[1, 2, 3], [4, 5, 6]]
        enf clk' = clk + a[2][0]
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}
