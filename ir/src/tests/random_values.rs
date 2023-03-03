use super::{parse, AirIR};

#[test]
fn random_values_indexed_access() {
    let source = "
    trace_columns:
        main: [a, b[12]]
        aux: [c, d]
    public_inputs:
        stack_inputs: [16]
    random_values:
        rand: [16]
    boundary_constraints:
        enf c.first = $rand[10] * 2
        enf c.last = 1
    integrity_constraints:
        enf c' = $rand[3] + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn random_values_custom_name() {
    let source = "
    trace_columns:
        main: [a, b[12]]
        aux: [c, d]
    public_inputs:
        stack_inputs: [16]
    random_values:
        alphas: [16]
    boundary_constraints:
        enf c.first = $alphas[10] * 2
        enf c.last = 1
    integrity_constraints:
        enf c' = $alphas[3] + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn random_values_named_access() {
    let source = "
    trace_columns:
        main: [a, b[12]]
        aux: [c, d]
    public_inputs:
        stack_inputs: [16]
    random_values:
        rand: [m, n[4]]
    boundary_constraints:
        enf c.first = (n[1] - $rand[0]) * 2
        enf c.last = m
    integrity_constraints:
        enf c' = m + n[2] + $rand[1]";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_ok());
}

#[test]
fn err_random_values_out_of_bounds() {
    let source_fixed_list = "
    trace_columns:
        main: [a, b[12]]
        aux: [c, d]
    public_inputs:
        stack_inputs: [16]
    random_values:
        rand: [4]
    boundary_constraints:
        enf a.first = $rand[10] * 2
        enf a.last = 1
    integrity_constraints:
        enf a' = $rand[4] + 1";

    let parsed = parse(source_fixed_list).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_err());

    let source_ident_vector_sub_vec = "
    trace_columns:
        main: [a, b[12]]
        aux: [c, d]
    public_inputs:
        stack_inputs: [16]
    random_values:
        rand: [m, n[4]]
    boundary_constraints:
        enf a.first = n[5] * 2
        enf a.last = 1
    integrity_constraints:
        enf a' = m + 1";

    let parsed = parse(source_ident_vector_sub_vec).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_err());

    let source_ident_vector_index = "
    trace_columns:
        main: [a, b[12]]
        aux: [c, d]
    public_inputs:
        stack_inputs: [16]
    random_values:
        rand: [m, n[4]]
    boundary_constraints:
        enf a.first = $rand[10] * 2
        enf a.last = 1
    integrity_constraints:
        enf a' = m + 1";

    let parsed = parse(source_ident_vector_index).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_err());
}

#[test]
fn err_random_values_without_aux_cols() {
    let source = "
    trace_columns:
        main: [a, b[12]]
    public_inputs:
        stack_inputs: [16]
    random_values:
        rand: [16]
    boundary_constraints:
        enf a.first = 2
        enf a.last = 1
    integrity_constraints:
        enf a' = a + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    assert!(result.is_err());
}

#[test]
fn err_random_values_in_bc_against_main_cols() {
    let source = "
    trace_columns:
        main: [a, b[12]]
        aux: [c, d]
    public_inputs:
        stack_inputs: [16]
    random_values:
        rand: [16]
    boundary_constraints:
        enf a.first = $rand[10] * 2
        enf b[2].last = 1
    integrity_constraints:
        enf c' = $rand[3] + 1";

    let parsed = parse(source).expect("Parsing failed");
    let result = AirIR::new(parsed);
    println!("{result:?}");
    assert!(result.is_err());
}
