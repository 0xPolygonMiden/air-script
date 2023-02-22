use crate::lexer::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    ScanError(Span),
    ParseError(ParseError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    InvalidConst(String),
    InvalidInt(String),
    InvalidListComprehension(String),
    InvalidRandomValues(String),
    InvalidTraceCols(String),
    MissingBoundaryConstraint(String),
    MissingIntegrityConstraint(String),
    MissingMainTraceCols(String),
}
