use super::{build_parse_test, Error, ParseError};

// TODO: clean up this test file
// IDENTIFIERS
// ================================================================================================

#[test]
fn error_invalid_int() {
    let num: u128 = u64::max_value() as u128 + 1;
    let source = format!(
        "
    transition_constraints:
        enf clk' = clk + {}",
        num
    );
    // Integers can only be of type u64.
    let error = Error::ParseError(ParseError::InvalidInt(format!("Int too big : {}", num)));
    build_parse_test!(source.as_str()).expect_error(error);
}

// UNRECOGNIZED TOKEN ERRORS
// ================================================================================================

#[test]
fn error_constraint_without_section() {
    // Constraints outside of valid sections are not allowed.
    let source = "enf clk' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn error_identifier_starting_with_int() {
    // Identifiers cannot start with numeric characters.
    // lexer considers the integer 1 and alphabetic clk' to be separate tokens
    // hence this fails at parser level since a valid identifier is expected
    // at that position which 1 is not.
    let source = "
    transition_constraints:
        enf 1clk' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();
}
