use super::build_parse_test;

// SECTIONS
// ================================================================================================

#[test]
fn error_constraint_without_section() {
    // Constraints outside of valid sections are not allowed.
    let source = "enf clk' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();
}
