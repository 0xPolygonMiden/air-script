use super::{expect_valid_tokenization, Token};

#[test]
fn random_values_empty_list() {
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
fn random_values_fixed_list() {
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
fn random_values_ident_vector() {
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

#[test]
fn random_values_index_access() {
    let source = "
    integrity_constraints:
        enf a + $alphas[1] = 0";

    let tokens = vec![
        Token::IntegrityConstraints,
        Token::Colon,
        Token::Enf,
        Token::Ident("a".to_string()),
        Token::Plus,
        Token::DeclIdentRef("$alphas".to_string()),
        Token::Lsqb,
        Token::Num("1".to_string()),
        Token::Rsqb,
        Token::Equal,
        Token::Num("0".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}
