#[macro_use]
extern crate lalrpop_util;

pub mod ast;

mod error;
use error::Error;

mod lexer;
use lexer::{Lexer, Token};

mod parser;
use crate::parser::SourceParser;

/// Parses the provided source and returns the AST.
pub fn parse(source: &str) -> Result<ast::Source, lalrpop_util::ParseError<usize, Token, Error>> {
    let lex = Lexer::new(source).spanned().map(Token::to_spanned);
    SourceParser::new().parse(lex)
}
