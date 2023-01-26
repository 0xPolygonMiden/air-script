use super::{expect_valid_tokenization, Token};

#[test]
fn random_values_section() {
    let source = "random_values";
    let tokens = vec![Token::RandomValues];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn random_values_empty_list_default_name() {
    let source = "
random_values:
    rand: []";

    let tokens = vec![
        Token::RandomValues,
        Token::Colon,
        Token::Ident("rand".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn random_values_fixed_list_default_name() {
    let source = "
random_values:
    rand: [15]";

    let tokens = vec![
        Token::RandomValues,
        Token::Colon,
        Token::Ident("rand".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Num("15".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn random_values_fixed_list_custom_name() {
    let source = "
random_values:
    alphas: [15]";

    let tokens = vec![
        Token::RandomValues,
        Token::Colon,
        Token::Ident("alphas".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Num("15".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn random_values_ident_vector_default_name() {
    let source = "
random_values:
    rand: [a, b[12], c]";

    let tokens = vec![
        Token::RandomValues,
        Token::Colon,
        Token::Ident("rand".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Ident("a".to_string()),
        Token::Comma,
        Token::Ident("b".to_string()),
        Token::Lsqb,
        Token::Num("12".to_string()),
        Token::Rsqb,
        Token::Comma,
        Token::Ident("c".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}
