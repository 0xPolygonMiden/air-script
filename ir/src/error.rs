#[derive(Debug)]
pub enum SemanticError {
    InvalidIdentifier(String),
    RedefinedBoundary(String),
    DuplicateTraceColumn(String),
    UndefinedIdentifier(String),
}
