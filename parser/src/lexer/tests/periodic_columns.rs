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
        Token::Lsqb,
        Token::Num("1".to_string()),
        Token::Comma,
        Token::Num("0".to_string()),
        Token::Comma,
        Token::Num("0".to_string()),
        Token::Comma,
        Token::Num("0".to_string()),
        Token::Rsqb,
        Token::Ident("k1".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Num("0".to_string()),
        Token::Comma,
        Token::Num("0".to_string()),
        Token::Comma,
        Token::Num("0".to_string()),
        Token::Comma,
        Token::Num("0".to_string()),
        Token::Comma,
        Token::Num("0".to_string()),
        Token::Comma,
        Token::Num("0".to_string()),
        Token::Comma,
        Token::Num("0".to_string()),
        Token::Comma,
        Token::Num("1".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}
