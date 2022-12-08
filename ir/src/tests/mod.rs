use crate::AirIR;
use parser::parse;

#[test]
fn boundary_constraints() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    transition_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn boundary_constraints_with_constants() {
    let source = "
    constants:
        A: 123
        B: [1, 2, 3]
        C: [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    transition_constraints:
        enf clk' = clk - 1
    boundary_constraints:
        enf clk.first = A
        enf clk.last = B[0] + C[0][1]";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn err_tc_invalid_vector_access() {
    let source = "
    constants:
        A: 123
        B: [1, 2, 3]
        C: [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    transition_constraints:
        enf clk' = clk + A + B[3] - C[1][2]";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_tc_invalid_matrix_access() {
    let source = "
    constants:
        A: 123
        B: [1, 2, 3]
        C: [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    transition_constraints:
        enf clk' = clk + A + B[1] - C[3][2]";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());

    let source = "
    constants:
        A: 123
        B: [1, 2, 3]
        C: [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    transition_constraints:
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
    transition_constraints:
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
    transition_constraints:
        enf clk' = clk + 1";

    assert!(parse(source).is_err());

    // if boundary constraints are omitted, an error should be returned at IR level.
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    transition_constraints:
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
    transition_constraints:
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
    transition_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.last = 0
        enf clk.last = 1";

    let parsed = parse(source).expect("Parsing failed");

    assert!(AirIR::from_source(&parsed).is_err());
}

#[test]
fn transition_constraints() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    transition_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn transition_constraints_with_constants() {
    let source = "
    constants:
        A: 123
        B: [1, 2, 3]
        C: [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
    transition_constraints:
        enf clk' = clk + A + B[1] - C[1][2]";

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
    transition_constraints:
        enf clk' = (clk + 1)";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn err_bc_invalid_vector_access() {
    let source = "
    constants:
        A: 123
        B: [1, 2, 3]
        C: [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = A + B[3] - C[1][2]
    transition_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_bc_invalid_matrix_access() {
    let source = "
    constants:
        A: 123
        B: [1, 2, 3]
        C: [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = A + B[1] - C[3][2]
    transition_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());

    let source = "
    constants:
        A: 123
        B: [1, 2, 3]
        C: [[1, 2, 3], [4, 5, 6]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = A + B[1] - C[1][3]
    transition_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}

#[test]
fn err_tc_empty_or_omitted() {
    // if transition constraints are empty, an error should be returned at parser level.
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    transition_constraints:
    boundary_constraints:
        enf clk.first = 0";

    assert!(parse(source).is_err());

    // if transition constraints are omitted, an error should be returned at IR level.
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
fn err_tc_column_undeclared() {
    let source = "
    trace_columns:
        main: [ctx]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf ctx.first = 0
    transition_constraints:
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
    transition_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = 0";

    assert!(parse(source).is_err());

    // if public inputs are omitted, an error should be returned at IR level.
    let source = "
    trace_columns:
        main: [clk]
    transition_constraints:
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
    transition_constraints:
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
    transition_constraints:
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
    transition_constraints:
        enf clk'^2 - clk = 1";
    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::from_source(&parsed);
    assert!(result.is_ok());
}

#[test]
fn err_invalid_matrix_const() {
    let source = "
    constants:
        A: [[2, 3], [1, 0, 2]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    transition_constraints:
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::from_source(&parsed);
    assert!(result.is_err());
}
