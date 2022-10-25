use super::expect_valid_tokenization;
use crate::lexer::Token;

// KEYWORDS VALID TOKENIZATION
// ================================================================================================

#[test]
fn valid_tokenization_next_token() {
    let source = "enf clk'' = clk + 1";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Next,
        // This is a parsing error, not a scanning error.
        Token::Next,
        Token::Equal,
        Token::Ident("clk".to_string()),
        Token::Plus,
        Token::Num("1".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}
