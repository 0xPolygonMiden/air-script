use super::expect_valid_tokenization;
use crate::lexer::Token;

// CONSTRAINTS VALID TOKENIZATION
// ================================================================================================

#[test]
fn boundary_constraints() {
    let source = "enf clk.first = 0";
    let tokens = vec![
        Token::Enf,
        Token::Ident("clk".to_string()),
        Token::Dot,
        Token::First,
        Token::Equal,
        Token::Num("0".to_string()),
    ];
    expect_valid_tokenization(source, tokens);
}
