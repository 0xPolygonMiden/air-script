use crate::lexer::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    ScanError(Span),
    ParseError(ParseError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    InvalidConst(String),
    InvalidEvaluatorFunction(String),
    InvalidInt(String),
    InvalidListComprehension(String),
    InvalidRandomValues(String),
    InvalidTraceCols(String),
    MissingBoundaryConstraint(String),
    MissingIntegrityConstraint(String),
    MissingMainTraceCols(String),
}

impl ParseError {
    pub fn missing_integrity_constraint() -> Self {
        ParseError::MissingIntegrityConstraint(
            "Declaration of at least one integrity constraint is required".to_string(),
        )
    }
}
