use super::{expect_valid_tokenization, Token};

#[test]
fn periodic_columns_kw() {
    let source = "periodic_columns";
    let tokens = vec![Token::PeriodicColumns];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn periodic_columns() {
    let source = "
periodic_columns:
    k0: [1, 0, 0, 0]
    k1: [0, 0, 0, 0, 0, 0, 0, 1]";

    let tokens = vec![
        Token::PeriodicColumns,
        Token::Colon,
        Token::Ident("k0".to_string()),
        Token::Colon,
        Token::LBracket,
        Token::Num(1),
        Token::Comma,
        Token::Num(0),
        Token::Comma,
        Token::Num(0),
        Token::Comma,
        Token::Num(0),
        Token::RBracket,
        Token::Ident("k1".to_string()),
        Token::Colon,
        Token::LBracket,
        Token::Num(0),
        Token::Comma,
        Token::Num(0),
        Token::Comma,
        Token::Num(0),
        Token::Comma,
        Token::Num(0),
        Token::Comma,
        Token::Num(0),
        Token::Comma,
        Token::Num(0),
        Token::Comma,
        Token::Num(0),
        Token::Comma,
        Token::Num(1),
        Token::RBracket,
    ];
    expect_valid_tokenization(source, tokens);
}
