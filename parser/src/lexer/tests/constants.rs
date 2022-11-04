use super::{expect_valid_tokenization, Token};

#[test]
fn constants_scalar() {
    let source = "
constants:
    a: 1
    b: 2";

    let tokens = vec![
        Token::Constants,
        Token::Colon,
        Token::Ident("a".to_string()),
        Token::Colon,
        Token::Num("1".to_string()),
        Token::Ident("b".to_string()),
        Token::Colon,
        Token::Num("2".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn constants_vector() {
    let source = "
constants:
    a: [1, 2, 3, 4]
    b: [5, 6, 7, 8]";

    let tokens = vec![
        Token::Constants,
        Token::Colon,
        Token::Ident("a".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Num("1".to_string()),
        Token::Comma,
        Token::Num("2".to_string()),
        Token::Comma,
        Token::Num("3".to_string()),
        Token::Comma,
        Token::Num("4".to_string()),
        Token::Rsqb,
        Token::Ident("b".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Num("5".to_string()),
        Token::Comma,
        Token::Num("6".to_string()),
        Token::Comma,
        Token::Num("7".to_string()),
        Token::Comma,
        Token::Num("8".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn constants_matrix() {
    let source = "
    constants:
        a: [[1, 2], [3, 4]]
        b: [[5, 6], [7, 8]]";

    let tokens = vec![
        Token::Constants,
        Token::Colon,
        Token::Ident("a".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Lsqb,
        Token::Num("1".to_string()),
        Token::Comma,
        Token::Num("2".to_string()),
        Token::Rsqb,
        Token::Comma,
        Token::Lsqb,
        Token::Num("3".to_string()),
        Token::Comma,
        Token::Num("4".to_string()),
        Token::Rsqb,
        Token::Rsqb,
        Token::Ident("b".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Lsqb,
        Token::Num("5".to_string()),
        Token::Comma,
        Token::Num("6".to_string()),
        Token::Rsqb,
        Token::Comma,
        Token::Lsqb,
        Token::Num("7".to_string()),
        Token::Comma,
        Token::Num("8".to_string()),
        Token::Rsqb,
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}
