use expect_test::expect_file;

use super::helpers::{Target, Test};

#[test]
fn binary() {
    let generated_air = Test::new("tests/binary/binary.air".to_string())
        .transpile(Target::Plonky3)
        .unwrap();

    let expected = expect_file!["../binary/binary_plonky3.rs"];
    expected.assert_eq(&generated_air);
}

// TODO: add all tests
