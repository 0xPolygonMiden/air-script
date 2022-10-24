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
        Token::Number("1".to_string()),
        Token::Comma,
        Token::Number("0".to_string()),
        Token::Comma,
        Token::Number("0".to_string()),
        Token::Comma,
        Token::Number("0".to_string()),
        Token::Rsqb,
        Token::Ident("k1".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Number("0".to_string()),
        Token::Comma,
        Token::Number("0".to_string()),
        Token::Comma,
        Token::Number("0".to_string()),
        Token::Comma,
        Token::Number("0".to_string()),
        Token::Comma,
        Token::Number("0".to_string()),
        Token::Comma,
        Token::Number("0".to_string()),
        Token::Comma,
        Token::Number("0".to_string()),
        Token::Comma,
        Token::Number("1".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}

// #[test]
// fn error_periodic_columns_length() {
//     let source = "
// periodic_columns:
//     k0: [1, 0, 0]";

//     let tokens = vec![
//         Token::PeriodicColumns,
//         Token::Colon,
//         Token::Ident("program_hash".to_string()),
//         Token::Colon,
//         Token::Lsqb,
//         Token::Number("4".to_string()),
//         Token::Rsqb,
//         Token::Ident("stack_inputs".to_string()),
//         Token::Colon,
//         Token::Lsqb,
//         Token::Number("12".to_string()),
//         Token::Rsqb,
//     ];
//     expect_valid_tokenization(source, tokens);
// }
