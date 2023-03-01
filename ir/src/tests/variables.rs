use super::{parse, AirIR};

#[test]
fn bc_scalar_variable() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        let a = 1 + 8
        enf clk.first = a
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn bc_vector_variable() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        let b = [1, 5]
        enf clk.first = b[0]
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn bc_with_variables() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        let a = 1 + 8
        let b = [a, a*a]
        enf clk.first = a + b[0]

        let c = [[b[0], b[1]], [clk, 2^2]]
        enf clk.last = c[1][1]

    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn bc_variable_in_both_domains() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        let a = 1 + 8
        enf clk.first = a
        enf clk.last = a
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(parsed);
    println!("{result:?}");
    assert!(result.is_ok());
}

#[test]
fn err_bc_variable_ref_next() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        let a = clk'
        enf clk.first = 0
        enf clk.last = a
    integrity_constraints:
        enf clk' = clk + 1";

    assert!(parse(source).is_err());
}

#[test]
fn ic_with_variables() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1
    integrity_constraints:
        let a = 1
        let b = [a, a*a]
        let c = [[clk' - clk, clk - a], [1 + 8, 2^2]]
        enf c[0][0] = 1";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}
#[test]
fn ic_variable_rebind() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 7
        enf clk.last = 8
    integrity_constraints:
        let a = [[1, 2], [3, 4]]
        let b = a[1]
        let c = b
        enf clk' = c[0]";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(parsed);
    println!("{result:?}");
    assert!(result.is_ok());
}

#[test]
fn ic_variables_access_vector_from_matrix() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 7
        enf clk.last = 8
    integrity_constraints:
        let a = [[1, 2], [3, 4]]
        let b = a[1]
        let c = [a[0], a[1], b]
        enf clk' = c[2][0] + c[0][1]";

    let parsed = parse(source).expect("Parsing failed");

    let result = AirIR::new(parsed);
    println!("{result:?}");
    assert!(result.is_ok());
}

#[test]
fn err_ic_variables_vector_with_inlined_vector() {
    // We can not parse matrix variable that consists of inlined vector and scalar elements.
    // Variable `d` is parsed as a vector and can not contain inlined vectors.
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 7
        enf clk.last = 8
    integrity_constraints:
        let a = [[1, 2], [3, 4]]
        let d = [a[0], [3, 4]]
        enf clk' = d[0][0]";

    parse(source).expect_err("Parsing failed");
}

#[test]
fn err_ic_variables_matrix_with_vector_reference() {
    // We can not parse matrix variable that consists of inlined vector and scalar elements
    // Variable `d` is parsed as a matrix and can not contain references to vectors.
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        enf clk.first = 7
        enf clk.last = 8
    integrity_constraints:
        let a = [[1, 2], [3, 4]]
        let d = [[3, 4], a[0]]
        enf clk' = d[0][0]";

    parse(source).expect_err("Parsing failed");
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
    let result = AirIR::new(parsed);
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
    let result = AirIR::new(parsed);
    assert!(result.is_err());
}

#[test]
fn err_variable_def_in_other_section() {
    let source = "
    const A = [[2, 3], [1, 0]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        let a = 1
        enf clk.first = 0
        enf clk.last = 1
    integrity_constraints:
        enf clk' = clk + a";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    println!("{result:?}");
    assert!(result.is_err());

    let source = "
    const A = [[2, 3], [1, 0]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    integrity_constraints:
        let a = 1
        enf clk' = clk + 1
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = a";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_err());
}

#[test]
fn err_bc_variable_vector_invalid_access() {
    let source = "
    const A = [[2, 3], [1, 0]]
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        let a = [1, 2]
        enf clk.first = a[2]
        enf clk.last = 1
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_err());
}

#[test]
fn err_ic_variable_vector_invalid_access() {
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
    let result = AirIR::new(parsed);
    assert!(result.is_err());
}

#[test]
fn err_bc_variable_matrix_invalid_access() {
    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        let a = [[1, 2, 3], [4, 5, 6]]
        enf clk.first = a[1][3]
        enf clk.last = 1
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_err());

    let source = "
    trace_columns:
        main: [clk]
    public_inputs:
        stack_inputs: [16]
    boundary_constraints:
        let a = [[1, 2, 3], [4, 5, 6]]
        enf clk.first = 0
        enf clk.last = a[2][0]
    integrity_constraints:
        enf clk' = clk + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    println!("{result:?}");
    assert!(result.is_err());
}

#[test]
fn err_ic_variable_matrix_invalid_access() {
    let source = "
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
    let result = AirIR::new(parsed);
    assert!(result.is_err());

    let source = "
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
    let result = AirIR::new(parsed);
    assert!(result.is_err());
}
