#[derive(Debug)]
pub enum SemanticError {
    DuplicateIdentifier(String),
    IndexOutOfRange(String),
    InvalidConstraint(String),
    InvalidIdentifier(String),
    InvalidPeriodicColumn(String),
    InvalidUsage(String),
    MissingDeclaration(String),
    TooManyConstraints(String),
}
