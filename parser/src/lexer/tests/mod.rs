use crate::{
    error::Error,
    lexer::{Lexer, Token},
};

mod arithmetic_ops;
mod boundary_constraints;
mod constants;
mod identifiers;
mod list_comprehension;
mod periodic_columns;
mod pub_inputs;
mod variables;

// TEST HELPERS
// ================================================================================================

fn expect_valid_tokenization(source: &str, expected_tokens: Vec<Token>) {
    let tokens: Vec<Token> = Lexer::new(source).collect();
    assert_eq!(tokens, expected_tokens);
}

/// Checks that source is valid and asserts that appropriate error is returned if there
/// is a problem while scanning the source.
fn expect_error(source: &str, error: Error) {
    let mut tokens = Lexer::new(source)
        .spanned()
        .map(Token::to_spanned)
        .peekable();

    while tokens.next_if(|token| token.is_ok()).is_some() {}
    let err = tokens.next();

    assert_eq!(err.unwrap().expect_err("No scan error"), error);
}
