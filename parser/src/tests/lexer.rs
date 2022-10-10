use crate::{
    ast::{AirDef, Identifier, Source, SourceSection},
    build_parse_test,
    error::Error,
    lexer::Token,
    tests::ParseTest,
};

// VALID TOKENIZATION
// ================================================================================================

#[test]
fn keywords_with_identifiers() {
    let source = "enf clk' = clk + 1";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk'".to_string()),
        Token::Equal,
        Token::Ident("clk".to_string()),
        Token::Plus,
        Token::Number("1".to_string()),
    ];
    build_parse_test!(source).expect_valid_tokenization(tokens);
}

#[test]
fn multi_arithmetic_ops() {
    let source = "enf clk' - clk - 1 = 0";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk'".to_string()),
        Token::Minus,
        Token::Ident("clk".to_string()),
        Token::Minus,
        Token::Number("1".to_string()),
        Token::Equal,
        Token::Number("0".to_string()),
    ];
    build_parse_test!(source).expect_valid_tokenization(tokens);
}

#[test]
fn boundary_constraints() {
    let source = "enf clk.first = 0";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::First,
        Token::Equal,
        Token::Number("0".to_string()),
    ];
    build_parse_test!(source).expect_valid_tokenization(tokens);
}

#[test]
fn number_and_ident_without_space() {
    let source = "enf 1clk' = clk + 1";
    let tokens = vec![
        Token::Enf,
        Token::Number("1".to_string()),
        Token::Ident("clk'".to_string()),
        Token::Equal,
        Token::Ident("clk".to_string()),
        Token::Plus,
        Token::Number("1".to_string()),
    ];
    build_parse_test!(source).expect_valid_tokenization(tokens);
}

#[test]
fn keyword_and_ident_without_space() {
    let source = "enfclk' = clkdef + 1";
    let tokens = vec![
        // enfclk' is considered as an identifier by logos
        Token::Ident("enfclk'".to_string()),
        Token::Equal,
        // clkdef is considered as an identifier by logos
        Token::Ident("clkdef".to_string()),
        Token::Plus,
        Token::Number("1".to_string()),
    ];
    build_parse_test!(source).expect_valid_tokenization(tokens);
}

// SCAN ERRORS
// ================================================================================================

#[test]
fn error_invalid_token() {
    let source = "enf clk'' = clk + 1";
    // Only one "'" is allowed at the end of identifier to indicate next row of the column.
    // Should fail if more than one "'" is present in an identifier
    let error = Error::ScanError(8..9);
    build_parse_test!(source).expect_error(error);
}

#[test]
fn error_identifier_with_invalid_characters() {
    let source = "enf clk@' = clk + 1";
    // "@" is not in the allowed characters.
    let expected = Error::ScanError(7..8);
    build_parse_test!(source).expect_error(expected);
}

#[test]
fn return_first_invalid_character_error() {
    let source = "enf clk@' = clk@ + 1";
    // "@" is not in the allowed characters.
    let expected = Error::ScanError(7..8);
    build_parse_test!(source).expect_error(expected);
}

#[test]
fn error_invalid_symbol() {
    let source = "enf clk' = clk / 1";
    // "/" is not a valid token.
    let expected = Error::ScanError(15..16);
    build_parse_test!(source).expect_error(expected);
}

// COMMENTS
// ================================================================================================

#[test]
fn simple_comment() {
    let source = "# Simple Comment";
    let expected = Source(vec![]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn inline_comment() {
    let source = "def SystemAir # Simple Comment";
    let expected = Source(vec![SourceSection::AirDef(AirDef {
        name: Identifier {
            name: "SystemAir".to_string(),
        },
    })]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multiline_comments() {
    let source = "# Comment line 1
    # Comment line 2
    def SystemAir";
    let expected = Source(vec![SourceSection::AirDef(AirDef {
        name: Identifier {
            name: "SystemAir".to_string(),
        },
    })]);
    build_parse_test!(source).expect_ast(expected);
}
