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
fn winterfell_variables() {
    let test = Test::new("tests/variables/variables.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["variables/variables.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn winterfell_trace_col_groups() {
    let test = Test::new("tests/trace_col_groups/trace_col_groups.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["trace_col_groups/trace_col_groups.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_trace_col_groups() {
    let test_path = "tests/trace_col_groups/trace_col_groups";
    let result_file = "tests/trace_col_groups/generated_trace_col_groups.json";

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
fn winterfell_indexed_trace_access() {
    let test = Test::new("tests/indexed_trace_access/indexed_trace_access.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["indexed_trace_access/indexed_trace_access.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_indexed_trace_access() {
    let test_path = "tests/indexed_trace_access/indexed_trace_access";
    let result_file = "tests/indexed_trace_access/generated_indexed_trace_access.json";

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
fn winterfell_random_values() {
    let test = Test::new("tests/random_values/random_values_simple.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["random_values/random_values.rs"];
    expected.assert_eq(&generated_air);

    let test = Test::new("tests/random_values/random_values_bindings.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["random_values/random_values.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn gce_random_values() {
    let test_path = "tests/random_values/random_values";
    let result_file = "tests/random_values/generated_random_values.json";

    let test_path_simple = &[test_path, "simple"].join("_");
    let test = Test::new([test_path_simple, "air"].join("."));
    test.generate_gce(2, result_file)
        .expect("GCE generation failed");

    let expected = expect_file![[test_path, "json"].join(".").trim_start_matches("tests/")];

    let mut file = File::open(result_file).expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read form file");

    expected.assert_eq(&contents);

    fs::remove_file(result_file).expect("Failed to remove file");

    let test_path_bindings = &[test_path, "simple"].join("_");
    let test = Test::new([test_path_bindings, "air"].join("."));
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
fn list_comprehension() {
    // TODO: Improve this test to include more complicated expressions
    let test = Test::new("tests/list_comprehension/list_comprehension.air".to_string());
    let generated_air = test
        .generate_winterfell()
        .expect("Failed to generate a Winterfell Air implementation");

    let expected = expect_file!["list_comprehension/list_comprehension.rs"];
    expected.assert_eq(&generated_air);
}
