use crate::lexer::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    ScanError(Span),
    ParseError(ParseError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    InvalidInt(String),
    InvalidTraceCols(String),
    MissingMainTraceCols(String),
    InvalidConst(String),
    InvalidListComprehension(String),
    MissingBoundaryConstraint(String),
    MissingIntegrityConstraint(String),
}
