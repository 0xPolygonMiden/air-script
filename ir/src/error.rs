#[derive(Debug)]
pub enum SemanticError {
    InvalidIdentifier(String),
    DuplicateIdentifier(String),
    InvalidUsage(String),
    IndexOutOfRange(String),
    TooManyConstraints(String),
    InvalidPeriodicColumn(String),
    MissingSourceSection(String),
}
