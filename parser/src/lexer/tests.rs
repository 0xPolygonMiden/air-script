use crate::{
    error::Error,
    lexer::{Lexer, Token},
};

// TEST HELPERS
// ================================================================================================

fn expect_valid_tokenization(source: &str, expected_tokens: Vec<Token>) {
    let tokens: Vec<Token> = Lexer::new(source).collect();
    assert_eq!(tokens, expected_tokens);
}

/// Checks that source is valid and asserts that appropriate error is returned if there
/// is a problem while scanning the source.
fn expect_error(source: &str, error: Error) {
    let mut tokens = Lexer::new(source)
        .spanned()
        .map(Token::to_spanned)
        .peekable();

    while tokens.next_if(|token| token.is_ok()).is_some() {}
    let err = tokens.next();

    assert_eq!(err.unwrap().expect_err("No scan error"), error);
}

// VALID TOKENIZATION
// ================================================================================================

#[test]
fn keywords_with_identifiers() {
    let source = "enf clk' = clk + 1";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Next,
        Token::Equal,
        Token::Ident("clk".to_string()),
        Token::Plus,
        Token::Number("1".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn multi_arithmetic_ops() {
    let source = "enf clk' - clk - 1 = 0";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Next,
        Token::Minus,
        Token::Ident("clk".to_string()),
        Token::Minus,
        Token::Number("1".to_string()),
        Token::Equal,
        Token::Number("0".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
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
    expect_valid_tokenization(source, tokens);
}

#[test]
fn number_and_ident_without_space() {
    let source = "enf 1clk' = clk + 1";
    let tokens = vec![
        Token::Enf,
        Token::Number("1".to_string()),
        Token::Ident("clk".to_string()),
        Token::Next,
        Token::Equal,
        Token::Ident("clk".to_string()),
        Token::Plus,
        Token::Number("1".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn keyword_and_ident_without_space() {
    let source = "enfclk' = clkdef + 1";
    let tokens = vec![
        // enfclk' is considered as an identifier by logos
        Token::Ident("enfclk".to_string()),
        Token::Next,
        Token::Equal,
        // clkdef is considered as an identifier by logos
        Token::Ident("clkdef".to_string()),
        Token::Plus,
        Token::Number("1".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn valid_tokenization_next_token() {
    let source = "enf clk'' = clk + 1";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Next,
        // This is a parsing error, not a scanning error.
        Token::Next,
        Token::Equal,
        Token::Ident("clk".to_string()),
        Token::Plus,
        Token::Number("1".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

// SCAN ERRORS
// ================================================================================================

#[test]
fn error_identifier_with_invalid_characters() {
    let source = "enf clk@' = clk + 1";
    // "@" is not in the allowed characters.
    let expected = Error::ScanError(7..8);
    expect_error(source, expected);
}

#[test]
fn return_first_invalid_character_error() {
    let source = "enf clk@' = clk@ + 1";
    // "@" is not in the allowed characters.
    let expected = Error::ScanError(7..8);
    expect_error(source, expected);
}

#[test]
fn error_invalid_symbol() {
    let source = "enf clk' = clk / 1";
    // "/" is not a valid token.
    let expected = Error::ScanError(15..16);
    expect_error(source, expected);
}
