use super::ParseTest;

// SECTIONS
// ================================================================================================

#[test]
fn error_constraint_without_section() {
    // Constraints outside of valid sections are not allowed.
    let source = r#"
    def test

    enf clk' = clk + 1
    "#;
    ParseTest::new().expect_unrecognized_token(source);
}
