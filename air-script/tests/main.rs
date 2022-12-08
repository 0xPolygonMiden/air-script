use expect_test::expect_file;

mod helpers;
use helpers::Test;

// TESTS
// ================================================================================================

#[test]
fn aux_trace() {
    let generated_air = Test::new("tests/aux_trace/aux_trace.air".to_string())
        .transpile()
        .unwrap();

    let expected = expect_file!["aux_trace/aux_trace.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn binary() {
    let generated_air = Test::new("tests/binary/binary.air".to_string())
        .transpile()
        .unwrap();

    let expected = expect_file!["binary/binary.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn periodic_columns() {
    let generated_air = Test::new("tests/periodic_columns/periodic_columns.air".to_string())
        .transpile()
        .unwrap();

    let expected = expect_file!["periodic_columns/periodic_columns.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn pub_inputs() {
    let generated_air = Test::new("tests/pub_inputs/pub_inputs.air".to_string())
        .transpile()
        .unwrap();

    let expected = expect_file!["pub_inputs/pub_inputs.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn system() {
    let generated_air = Test::new("tests/system/system.air".to_string())
        .transpile()
        .unwrap();

    let expected = expect_file!["system/system.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn bitwise() {
    let generated_air = Test::new("tests/bitwise/bitwise.air".to_string())
        .transpile()
        .unwrap();

    let expected = expect_file!["bitwise/bitwise.rs"];
    expected.assert_eq(&generated_air);
}

#[test]
fn constants() {
    let generated_air = Test::new("tests/constants/constants.air".to_string())
        .transpile()
        .unwrap();

    let expected = expect_file!["constants/constants.rs"];
    expected.assert_eq(&generated_air);
}
