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
        Token::Num("1".to_string()),
        Token::Equal,
        Token::Num("0".to_string()),
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
        Token::Num("1".to_string()),
        Token::Equal,
        Token::Num("0".to_string()),
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
        Token::Mul,
        Token::Ident("clk".to_string()),
        Token::Mul,
        Token::Num("1".to_string()),
        Token::Equal,
        Token::Num("0".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn exp_op() {
    let source = "enf clk'^2 - clk - 1 = 0";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Next,
        Token::Exp,
        Token::Num("2".to_string()),
        Token::Minus,
        Token::Ident("clk".to_string()),
        Token::Minus,
        Token::Num("1".to_string()),
        Token::Equal,
        Token::Num("0".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn ops_with_parens() {
    let source = "enf clk' - (clk + 1) = 0";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Next,
        Token::Minus,
        Token::Lparen,
        Token::Ident("clk".to_string()),
        Token::Plus,
        Token::Num("1".to_string()),
        Token::Rparen,
        Token::Equal,
        Token::Num("0".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn ops_without_matching_closing_parens() {
    // This doesn't throw an error while scanning but while parsing.
    let source = "enf (clk' - (clk + 1) = 0";
    let tokens = vec![
        Token::Enf,
        Token::Lparen,
        Token::Ident("clk".to_string()),
        Token::Next,
        Token::Minus,
        Token::Lparen,
        Token::Ident("clk".to_string()),
        Token::Plus,
        Token::Num("1".to_string()),
        Token::Rparen,
        Token::Equal,
        Token::Num("0".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}
