use expect_test::expect_file;
use std::fs::{self, File};
use std::io::prelude::*;
mod helpers;
use helpers::Test;

// TESTS
// ================================================================================================

#[test]
fn winterfell_aux_trace() {
    let test = Test::new("tests/aux_trace/aux_trace.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["aux_trace/aux_trace.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_aux_trace() {
    let test_path = "tests/aux_trace/aux_trace";
    let result_file = "tests/aux_trace/generated_aux_trace.json";

    let test = Test::new([test_path, "air"].join("."));
    test.generate_gce(2, result_file)
        .expect("GCE generation failed");

    let expected = expect_file![[test_path, "json"].join(".").trim_start_matches("tests/")];

    let mut file = File::open(result_file).expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read form file");

    expected.assert_eq(&contents);

    fs::remove_file(result_file).expect("Failed to remove file");
}

#[test]
fn winterfell_binary() {
    let test = Test::new("tests/binary/binary.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["binary/binary.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_binary() {
    let test_path = "tests/binary/binary";
    let result_file = "tests/binary/generated_binary.json";

    let test = Test::new([test_path, "air"].join("."));
    test.generate_gce(2, result_file)
        .expect("GCE generation failed");

    let expected = expect_file![[test_path, "json"].join(".").trim_start_matches("tests/")];

    let mut file = File::open(result_file).expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read form file");

    expected.assert_eq(&contents);

    fs::remove_file(result_file).expect("Failed to remove file");
}

#[test]
fn winterfell_periodic_columns() {
    let test = Test::new("tests/periodic_columns/periodic_columns.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["periodic_columns/periodic_columns.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn winterfell_pub_inputs() {
    let test = Test::new("tests/pub_inputs/pub_inputs.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["pub_inputs/pub_inputs.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_pub_inputs() {
    let test_path = "tests/pub_inputs/pub_inputs";
    let result_file = "tests/pub_inputs/generated_pub_inputs.json";

    let test = Test::new([test_path, "air"].join("."));
    test.generate_gce(2, result_file)
        .expect("GCE generation failed");

    let expected = expect_file![[test_path, "json"].join(".").trim_start_matches("tests/")];

    let mut file = File::open(result_file).expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read form file");

    expected.assert_eq(&contents);

    fs::remove_file(result_file).expect("Failed to remove file");
}

#[test]
fn winterfell_system() {
    let test = Test::new("tests/system/system.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["system/system.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_system() {
    let test_path = "tests/system/system";
    let result_file = "tests/system/generated_system.json";

    let test = Test::new([test_path, "air"].join("."));
    test.generate_gce(2, result_file)
        .expect("GCE generation failed");

    let expected = expect_file![[test_path, "json"].join(".").trim_start_matches("tests/")];

    let mut file = File::open(result_file).expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read form file");

    expected.assert_eq(&contents);

    fs::remove_file(result_file).expect("Failed to remove file");
}

#[test]
fn winterfell_bitwise() {
    let test = Test::new("tests/bitwise/bitwise.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["bitwise/bitwise.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn winterfell_constants() {
    let test = Test::new("tests/constants/constants.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["constants/constants.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_constants() {
    let test_path = "tests/constants/constants";
    let result_file = "tests/constants/generated_constants.json";

    let test = Test::new([test_path, "air"].join("."));
    test.generate_gce(2, result_file)
        .expect("GCE generation failed");

    let expected = expect_file![[test_path, "json"].join(".").trim_start_matches("tests/")];

    let mut file = File::open(result_file).expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read form file");

    expected.assert_eq(&contents);

    fs::remove_file(result_file).expect("Failed to remove file");
}

#[test]
fn variables() {
    let test = Test::new("tests/variables/variables.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["variables/variables.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn trace_col_groups() {
    let test = Test::new("tests/trace_col_groups/trace_col_groups.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["trace_col_groups/trace_col_groups.rs"];
    expected.assert_eq(&generated_air);
}
