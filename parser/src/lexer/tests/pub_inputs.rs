use super::{expect_valid_tokenization, Token};

#[test]
fn pub_inputs_kw() {
    let source = "public_inputs";
    let tokens = vec![Token::PublicInputs];
    expect_valid_tokenization(source, tokens);
}

#[test]
fn pub_inputs_sized_arrays() {
    let source = "
public_inputs:
    program_hash: [4]
    stack_inputs: [12]";

    let tokens = vec![
        Token::PublicInputs,
        Token::Colon,
        Token::Ident("program_hash".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Number("4".to_string()),
        Token::Rsqb,
        Token::Ident("stack_inputs".to_string()),
        Token::Colon,
        Token::Lsqb,
        Token::Number("12".to_string()),
        Token::Rsqb,
    ];
    expect_valid_tokenization(source, tokens);
}
