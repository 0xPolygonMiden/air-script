use std::fs::File;
use std::io::prelude::*;

use expect_test::expect_file;
mod helpers;
use helpers::Test;

// TESTS
// ================================================================================================

#[test]
fn winterfell_aux_trace() {
    let generated_air = Test::new("tests/aux_trace/aux_trace.air".to_string())
        .unwrap()
        .generate_winterfell();

    let expected = expect_file!["aux_trace/aux_trace.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_aux_trace() {
    Test::new("tests/aux_trace/aux_trace.air".to_string())
        .unwrap()
        .generate_gce(2, "tests/aux_trace/generated_aux_trace.json")
        .unwrap();

    let expected = expect_file!["aux_trace/aux_trace.json"];

    let mut file = File::open("tests/aux_trace/generated_aux_trace.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    expected.assert_eq(&contents);
}

#[test]
fn winterfell_binary() {
    let generated_air = Test::new("tests/binary/binary.air".to_string())
        .unwrap()
        .generate_winterfell();

    let expected = expect_file!["binary/binary.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_binary() {
    Test::new("tests/binary/binary.air".to_string())
        .unwrap()
        .generate_gce(2, "tests/binary/generated_binary.json")
        .unwrap();

    let expected = expect_file!["binary/binary.json"];

    let mut file = File::open("tests/binary/generated_binary.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    expected.assert_eq(&contents);
}

#[test]
fn winterfell_periodic_columns() {
    let generated_air = Test::new("tests/periodic_columns/periodic_columns.air".to_string())
        .unwrap()
        .generate_winterfell();

    let expected = expect_file!["periodic_columns/periodic_columns.rs"];
    expected.assert_eq(&generated_air);
}

// not yet implemented (periodic columns)
#[test]
#[ignore]
fn gce_periodic_columns() {
    Test::new("tests/periodic_columns/periodic_columns.air".to_string())
        .unwrap()
        .generate_gce(2, "tests/periodic_columns/generated_periodic_columns.json")
        .unwrap();

    let expected = expect_file!["periodic_columns/periodic_columns.json"];

    let mut file = File::open("tests/periodic_columns/generated_periodic_columns.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    expected.assert_eq(&contents);
}

#[test]
fn winterfell_pub_inputs() {
    let generated_air = Test::new("tests/pub_inputs/pub_inputs.air".to_string())
        .unwrap()
        .generate_winterfell();

    let expected = expect_file!["pub_inputs/pub_inputs.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_pub_inputs() {
    Test::new("tests/pub_inputs/pub_inputs.air".to_string())
        .unwrap()
        .generate_gce(2, "tests/pub_inputs/generated_pub_inputs.json")
        .unwrap();

    let expected = expect_file!["pub_inputs/pub_inputs.json"];

    let mut file = File::open("tests/pub_inputs/generated_pub_inputs.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    expected.assert_eq(&contents);
}

#[test]
fn winterfell_system() {
    let generated_air = Test::new("tests/system/system.air".to_string())
        .unwrap()
        .generate_winterfell();

    let expected = expect_file!["system/system.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_system() {
    Test::new("tests/system/system.air".to_string())
        .unwrap()
        .generate_gce(2, "tests/system/generated_system.json")
        .unwrap();

    let expected = expect_file!["system/system.json"];

    let mut file = File::open("tests/system/generated_system.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    expected.assert_eq(&contents);
}

#[test]
fn winterfell_bitwise() {
    let generated_air = Test::new("tests/bitwise/bitwise.air".to_string())
        .unwrap()
        .generate_winterfell();

    let expected = expect_file!["bitwise/bitwise.rs"];
    expected.assert_eq(&generated_air);
}

// not yet implemented (periodic columns)
#[test]
#[ignore]
fn gce_bitwise() {
    Test::new("tests/bitwise/bitwise.air".to_string())
        .unwrap()
        .generate_gce(2, "tests/bitwise/generated_bitwise.json")
        .unwrap();

    let expected = expect_file!["bitwise/bitwise.json"];

    let mut file = File::open("tests/bitwise/generated_bitwise.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    expected.assert_eq(&contents);
}

#[test]
fn winterfell_constants() {
    let generated_air = Test::new("tests/constants/constants.air".to_string())
        .unwrap()
        .generate_winterfell();

    let expected = expect_file!["constants/constants.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_constants() {
    Test::new("tests/constants/constants.air".to_string())
        .unwrap()
        .generate_gce(2, "tests/constants/generated_constants.json")
        .unwrap();

    let expected = expect_file!["constants/constants.json"];

    let mut file = File::open("tests/constants/generated_constants.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    expected.assert_eq(&contents);
}

#[test]
#[ignore] // exponentiation for boundary constraints is not ready
fn gce_exponentiation() {
    Test::new("tests/exponentiation/exponentiation.air".to_string())
        .unwrap()
        .generate_gce(2, "tests/exponentiation/generated_exponentiation.json")
        .unwrap();

    let expected = expect_file!["exponentiation/exponentiation.json"];

    let mut file = File::open("tests/exponentiation/generated_exponentiation.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    expected.assert_eq(&contents);
}

#[test]
fn variables() {
    let generated_air = Test::new("tests/variables/variables.air".to_string())
        .unwrap()
        .generate_winterfell();

    let expected = expect_file!["variables/variables.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn trace_col_groups() {
    let generated_air = Test::new("tests/trace_col_groups/trace_col_groups.air".to_string())
        .unwrap()
        .generate_winterfell();

    let expected = expect_file!["trace_col_groups/trace_col_groups.rs"];
    expected.assert_eq(&generated_air);
}
