#[macro_use]
extern crate lalrpop_util;

pub mod ast;
mod lexer;
mod parser;
mod sema;
pub mod symbols;
pub mod transforms;

pub use self::parser::{ParseError, Parser};
pub use self::sema::{LexicalScope, SemanticAnalysisError};
pub use self::symbols::Symbol;

use std::path::Path;
use std::sync::Arc;

use miden_diagnostics::{CodeMap, DiagnosticsHandler};

/// Parses the provided source and returns the AST.
pub fn parse(
    diagnostics: &DiagnosticsHandler,
    codemap: Arc<CodeMap>,
    source: &str,
) -> Result<ast::Program, ParseError> {
    let parser = Parser::new((), codemap);
    match parser.parse_string::<ast::Program, _, _>(diagnostics, source) {
        Ok(ast) => Ok(ast),
        Err(ParseError::Lexer(err)) => {
            diagnostics.emit(err);
            Err(ParseError::Failed)
        }
        Err(err) => Err(err),
    }
}

/// Parses a [Module] from the given path.
///
/// This is primarily intended for use in the import resolution phase.
pub(crate) fn parse_module_from_file<P: AsRef<Path>>(
    diagnostics: &DiagnosticsHandler,
    codemap: Arc<CodeMap>,
    path: P,
) -> Result<ast::Module, ParseError> {
    let parser = Parser::new((), codemap);
    match parser.parse_file::<ast::Module, _, _>(diagnostics, path) {
        ok @ Ok(_) => ok,
        Err(ParseError::Lexer(err)) => {
            diagnostics.emit(err);
            Err(ParseError::Failed)
        }
        err @ Err(_) => err,
    }
}

/// Parses a [Module] from a file already in the codemap
///
/// This is primarily intended for use in the import resolution phase.
pub(crate) fn parse_module(
    diagnostics: &DiagnosticsHandler,
    codemap: Arc<CodeMap>,
    source: Arc<miden_diagnostics::SourceFile>,
) -> Result<ast::Module, ParseError> {
    let parser = Parser::new((), codemap);
    match parser.parse::<ast::Module, _>(diagnostics, source) {
        ok @ Ok(_) => ok,
        Err(ParseError::Lexer(err)) => {
            diagnostics.emit(err);
            Err(ParseError::Failed)
        }
        err @ Err(_) => err,
    }
}
