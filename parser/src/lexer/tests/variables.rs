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
        Token::Num(0),
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
        Token::LBracket,
        Token::Num(0),
        Token::Comma,
        Token::Num(1),
        Token::RBracket,
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::First,
        Token::Equal,
        Token::Ident("boundary_values".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::RBracket,
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::Last,
        Token::Equal,
        Token::Ident("boundary_values".to_string()),
        Token::LBracket,
        Token::Num(1),
        Token::RBracket,
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
        Token::Num(0),
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Quote,
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
        Token::LBracket,
        Token::Num(0),
        Token::Comma,
        Token::Num(1),
        Token::RBracket,
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Quote,
        Token::Equal,
        Token::Ident("clk".to_string()),
        Token::Minus,
        Token::Ident("a".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::RBracket,
        Token::Plus,
        Token::Ident("a".to_string()),
        Token::LBracket,
        Token::Num(1),
        Token::RBracket,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn variables_with_or_operators() {
    let source = "
    integrity_constraints:
        let flag = s[0] | !s[1]'
        enf clk' = clk + 1 when flag";
    let tokens = vec![
        Token::IntegrityConstraints,
        Token::Colon,
        Token::Let,
        Token::Ident("flag".to_string()),
        Token::Equal,
        Token::Ident("s".to_string()),
        Token::LBracket,
        Token::Num(0),
        Token::RBracket,
        Token::Bar,
        Token::Bang,
        Token::Ident("s".to_string()),
        Token::LBracket,
        Token::Num(1),
        Token::RBracket,
        Token::Quote,
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Quote,
        Token::Equal,
        Token::Ident("clk".to_string()),
        Token::Plus,
        Token::Num(1),
        Token::When,
        Token::Ident("flag".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}
