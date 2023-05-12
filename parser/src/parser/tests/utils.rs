use std::sync::Arc;

use miden_diagnostics::{CodeMap, DiagnosticsConfig, DiagnosticsHandler, Emitter, Verbosity};

use crate::{
    ast::Source,
    parser::{ParseError, Parser},
};

macro_rules! assert_matches {
    ($left:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
        match $left {
            $( $pattern )|+ $( if $guard )? => {}
            ref left_val => {
                panic!(r#"assertion failed: `(left matches right)`
                left: `{:?}`,
                right: `{:?}`"#,
                            left_val, stringify!($($pattern)|+ $(if $guard)?));
            }
        }
    }
}

struct SplitEmitter {
    capture: miden_diagnostics::CaptureEmitter,
    default: miden_diagnostics::DefaultEmitter,
}
impl SplitEmitter {
    #[inline]
    pub fn new() -> Self {
        use miden_diagnostics::term::termcolor::ColorChoice;

        Self {
            capture: Default::default(),
            default: miden_diagnostics::DefaultEmitter::new(ColorChoice::Auto),
        }
    }

    pub fn captured(&self) -> String {
        self.capture.captured()
    }
}
impl Emitter for SplitEmitter {
    #[inline]
    fn buffer(&self) -> miden_diagnostics::term::termcolor::Buffer {
        self.capture.buffer()
    }

    #[inline]
    fn print(&self, buffer: miden_diagnostics::term::termcolor::Buffer) -> std::io::Result<()> {
        use std::io::Write;

        let mut copy = self.capture.buffer();
        copy.write_all(buffer.as_slice())?;
        self.capture.print(buffer)?;
        self.default.print(copy)
    }
}

// TEST HANDLER
// ================================================================================================

/// [ParseTest] is a container for the data required to run parser tests. Used to build an AST from
/// the given source string and asserts that executing the test will result in the expected AST.
///
/// # Errors:
/// - ScanError test: check that the source provided contains valid characters and keywords.
/// - ParseError test: check that the parsed values are valid.
///   * InvalidInt: This error is returned if the parsed number is not a valid u64.
pub struct ParseTest {
    diagnostics: Arc<DiagnosticsHandler>,
    emitter: Arc<SplitEmitter>,
    parser: Parser,
}

impl ParseTest {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------

    /// Creates a new test, from the source string.
    pub fn new() -> Self {
        let codemap = Arc::new(CodeMap::new());
        let emitter = Arc::new(SplitEmitter::new());
        let config = DiagnosticsConfig {
            verbosity: Verbosity::Warning,
            warnings_as_errors: true,
            no_warn: false,
            display: Default::default(),
        };
        let diagnostics = Arc::new(DiagnosticsHandler::new(
            config,
            codemap.clone(),
            emitter.clone(),
        ));
        let parser = Parser::new((), codemap);
        Self {
            diagnostics,
            emitter,
            parser,
        }
    }

    pub fn parse_file(&self, path: &str) -> Result<Source, ParseError> {
        self.parser
            .parse_file::<Source, _, _>(&self.diagnostics, path)
    }

    pub fn parse(&self, source: &str) -> Result<Source, ParseError> {
        self.parser
            .parse_string::<Source, _, _>(&self.diagnostics, source)
    }

    // TEST METHODS
    // --------------------------------------------------------------------------------------------

    /// Checks that source is valid and asserts that appropriate error is returned if there
    /// is a problem while parsing the source.
    #[allow(unused)]
    #[track_caller]
    pub fn expect_error(&self, source: &str, expected: ParseError) {
        if let Err(err) = self.parse(source) {
            assert_eq!(err, expected)
        } else {
            panic!("expected parsing to fail, but it succeeded");
        }
    }

    #[track_caller]
    pub fn expect_diagnostic(&self, source: &str, expected: &str) {
        if let Err(err) = self.parse(source) {
            self.diagnostics.emit(err);
            assert!(
                self.emitter.captured().contains(expected),
                "expected diagnostic output to contain the string: '{}'",
                expected
            );
        } else {
            panic!("expected parsing to fail, but it succeeded");
        }
    }

    /// If an unrecognized token is present in the source string, return UnrecognizedToken error.
    #[track_caller]
    pub fn expect_unrecognized_token(&self, source: &str) {
        if let Err(err) = self.parse(source) {
            assert_matches!(err, ParseError::UnrecognizedToken { .. })
        } else {
            panic!("expected unrecognized token error, but parsing succeeded");
        }
    }

    /// Builds an AST from the given source string and asserts that executing the test will result
    /// in the expected AST.
    #[track_caller]
    pub fn expect_ast(&self, source: &str, expected: Source) {
        match self.parse(source) {
            Err(err) => {
                self.diagnostics.emit(err);
                panic!("expected parsing to succeed, see diagnostics for details");
            }
            Ok(ast) => assert_eq!(ast, expected),
        }
    }

    /// Builds an AST from the given source path and asserts that executing the test will result
    /// in the expected AST.
    #[track_caller]
    pub fn expect_ast_from_file(&self, path: &str, expected: Source) {
        match self.parse_file(path) {
            Err(err) => {
                self.diagnostics.emit(err);
                panic!("expected parsing to succeed, see diagnostics for details");
            }
            Ok(ast) => assert_eq!(ast, expected),
        }
    }
}
