use lalrpop_util::ParseError;

use crate::{
    ast::Source,
    error::Error,
    grammar,
    lexer::{Lexer, Token},
};

mod lexer;
mod parser;

// TEST HANDLER
// ================================================================================================

/// Returns a ParseTest struct from a source string.
///
/// Parameters:
/// * `source`: a source string representing a valid or invalid set of AIR constraints.
#[macro_export]
macro_rules! build_parse_test {
    ($source:expr) => {{
        ParseTest::new($source)
    }};
}

/// [ParseTest] is a container for the data required to run parser tests. Used to build an AST from
/// the given source string and asserts that executing the test will result in the expected AST.
///
/// # Errors:
/// - ScanError test: check that the source provided contains valid characters and keywords.
/// - ParseError test: check that the parsed values are valid.
///   * InvalidInt: This error is returned if the parsed number is not a valid u64.
pub struct ParseTest {
    pub source: String,
}

impl ParseTest {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------

    /// Creates a new test, from the source string.
    pub fn new(source: &str) -> Self {
        ParseTest {
            source: source.to_string(),
        }
    }

    // TEST METHODS
    // --------------------------------------------------------------------------------------------

    pub fn expect_valid_tokenization(&self, expected_tokens: Vec<Token>) {
        let tokens: Vec<Token> = Lexer::new(self.source.as_str()).collect();
        assert_eq!(tokens, expected_tokens);
    }

    /// Checks that source is valid and asserts that appropriate error is returned if there
    /// is a problem while scanning or parsing the source.
    pub fn expect_error(&self, error: Error) {
        let lex = Lexer::new(self.source.as_str())
            .spanned()
            .map(Token::to_spanned);
        let mut tokens = Lexer::new(self.source.as_str())
            .spanned()
            .map(Token::to_spanned)
            .peekable();
        while tokens.next_if(|token| token.is_ok()).is_some() {}
        let err = tokens.next();
        if err.is_some() {
            assert_eq!(err.unwrap().err().expect("No scan error"), error);
        } else {
            let source_parsed = grammar::SourceParser::new().parse(lex);
            let expected_error = Err(ParseError::User { error });
            assert_eq!(source_parsed, expected_error);
        }
    }

    /// If an unrecognized token is present in the source string, return UnrecognizedToken error.
    pub fn expect_unrecognized_token(&self) {
        let lex = Lexer::new(self.source.as_str())
            .spanned()
            .map(Token::to_spanned);
        let source_parsed = grammar::SourceParser::new().parse(lex);
        assert!(matches!(
            source_parsed,
            Err(ParseError::UnrecognizedToken { .. })
        ));
    }

    /// Builds an AST from the given source string and asserts that executing the test will result
    /// in the expected AST.
    pub fn expect_ast(&self, expected: Source) {
        let lex = Lexer::new(self.source.as_str())
            .spanned()
            .map(Token::to_spanned);
        let source_parsed = grammar::SourceParser::new().parse(lex).unwrap();
        assert_eq!(source_parsed, expected);
    }
}
