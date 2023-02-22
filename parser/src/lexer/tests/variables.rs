use super::{expect_valid_tokenization, Token};

// VARIABLES VALID TOKENIZATION
// ================================================================================================

#[test]
fn boundary_constraint_with_scalar_variables() {
    let source = "
    let first_value = 0
    enf clk.first = first_value";
    let tokens = vec![
        Token::Let,
        Token::Ident("first_value".to_string()),
        Token::Equal,
        Token::Num("0".to_string()),
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::First,
        Token::Equal,
        Token::Ident("first_value".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn boundary_constraint_with_vector_variables() {
    let source = "
    let boundary_values = [0, 1]
    enf clk.first = boundary_values[0]
    enf clk.last = boundary_values[1]";
    let tokens = vec![
        Token::Let,
        Token::Ident("boundary_values".to_string()),
        Token::Equal,
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Comma,
        Token::Num("1".to_string()),
        Token::Rsqb,
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::First,
        Token::Equal,
        Token::Ident("boundary_values".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::Last,
        Token::Equal,
        Token::Ident("boundary_values".to_string()),
        Token::Lsqb,
        Token::Num("1".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn integrity_constraint_with_scalar_variables() {
    let source = "
    let a = 0
    enf clk' = clk - a";
    let tokens = vec![
        Token::Let,
        Token::Ident("a".to_string()),
        Token::Equal,
        Token::Num("0".to_string()),
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Next,
        Token::Equal,
        Token::Ident("clk".to_string()),
        Token::Minus,
        Token::Ident("a".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn integrity_constraint_with_vector_variables() {
    let source = "
    let a = [0, 1]
    enf clk' = clk - a[0] + a[1]";
    let tokens = vec![
        Token::Let,
        Token::Ident("a".to_string()),
        Token::Equal,
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Comma,
        Token::Num("1".to_string()),
        Token::Rsqb,
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Next,
        Token::Equal,
        Token::Ident("clk".to_string()),
        Token::Minus,
        Token::Ident("a".to_string()),
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Rsqb,
        Token::Plus,
        Token::Ident("a".to_string()),
        Token::Lsqb,
        Token::Num("1".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}
