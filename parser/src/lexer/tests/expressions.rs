use super::expect_valid_tokenization;
use crate::lexer::Token;

// VALID TOKENIZATION
// ================================================================================================

#[test]
fn chained_add_ops() {
    let source = "enf clk' + clk + 1 = 0";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Next,
        Token::Plus,
        Token::Ident("clk".to_string()),
        Token::Plus,
        Token::Number("1".to_string()),
        Token::Equal,
        Token::Number("0".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn chained_sub_ops() {
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
fn chained_mul_ops() {
    let source = "enf clk' * clk * 1 = 0";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Next,
        Token::Multiply,
        Token::Ident("clk".to_string()),
        Token::Multiply,
        Token::Number("1".to_string()),
        Token::Equal,
        Token::Number("0".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}
