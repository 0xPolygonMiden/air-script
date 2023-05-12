#[macro_use]
extern crate lalrpop_util;

pub mod ast;

mod lexer;
mod parser;

pub use crate::parser::{ParseError, Parser};

use std::sync::Arc;

use miden_diagnostics::{CodeMap, DiagnosticsHandler};

/// Parses the provided source and returns the AST.
pub fn parse(
    diagnostics: &DiagnosticsHandler,
    codemap: Arc<CodeMap>,
    source: &str,
) -> Result<ast::Source, ParseError> {
    let parser = Parser::new((), codemap);
    match parser.parse_string::<ast::Source, _, _>(diagnostics, source) {
        Ok(ast) => Ok(ast),
        Err(ParseError::Lexer(err)) => {
            diagnostics.emit(err);
            Err(ParseError::Failed)
        }
        Err(err) => Err(err),
    }
}
