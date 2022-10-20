use lalrpop_util::ParseError;

use super::SourceParser;
use crate::{
    ast::Source,
    error::Error,
    lexer::{Lexer, Token},
};

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

    /// Checks that source is valid and asserts that appropriate error is returned if there
    /// is a problem while parsing the source.
    pub fn expect_error(&self, error: Error) {
        let lex = Lexer::new(self.source.as_str())
            .spanned()
            .map(Token::to_spanned);

        let source_parsed = SourceParser::new().parse(lex);
        let expected_error = Err(ParseError::User { error });
        assert_eq!(source_parsed, expected_error);
    }

    /// If an unrecognized token is present in the source string, return UnrecognizedToken error.
    pub fn expect_unrecognized_token(&self) {
        let lex = Lexer::new(self.source.as_str())
            .spanned()
            .map(Token::to_spanned);
        let source_parsed = SourceParser::new().parse(lex);
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
        let source_parsed = SourceParser::new().parse(lex).unwrap();
        assert_eq!(source_parsed, expected);
    }
}
