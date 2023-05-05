use crate::AirIR;

mod access;
mod boundary_constraints;
mod constant;
mod evaluators;
mod integrity_constraints;
mod list_folding;
mod pub_inputs;
mod random_values;
mod selectors;
mod source_sections;
mod trace;
mod variables;

use std::sync::Arc;

use miden_diagnostics::*;
use parser::{ast, ParseError, Parser};

/// Parses the provided source and returns the AST.
pub fn parse(source: &str) -> Result<ast::Source, ParseError> {
    use miden_diagnostics::term::termcolor::ColorChoice;

    let codemap = Arc::new(CodeMap::new());
    let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
    let config = DiagnosticsConfig {
        verbosity: Verbosity::Warning,
        warnings_as_errors: true,
        no_warn: false,
        display: Default::default(),
    };
    let diagnostics = DiagnosticsHandler::new(config, codemap.clone(), emitter);
    let parser = Parser::new((), codemap);
    match parser.parse_string::<ast::Source, _, _>(&diagnostics, source) {
        Ok(ast) => Ok(ast),
        Err(ParseError::Lexer(err)) => {
            diagnostics.emit(err);
            Err(ParseError::Failed)
        }
        Err(err) => Err(err),
    }
}
