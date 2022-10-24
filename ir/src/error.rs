#[derive(Debug)]
pub enum SemanticError {
    InvalidIdentifier(String),
    IndexOutOfRange(String),
    RedefinedBoundary(String),
    DuplicateTraceColumn(String),
    DuplicatePublicInput(String),
}
