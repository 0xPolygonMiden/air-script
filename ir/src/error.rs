#[derive(Debug)]
pub enum SemanticError {
    InvalidConstraint(String),
    InvalidIdentifier(String),
    DuplicateIdentifier(String),
    InvalidUsage(String),
    IndexOutOfRange(String),
    TooManyConstraints(String),
    InvalidPeriodicColumn(String),
    MissingDeclaration(String),
}
