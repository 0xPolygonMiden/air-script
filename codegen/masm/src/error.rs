#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("invalid access type")]
    InvalidAccessType,
    #[error("invalid row offset")]
    InvalidRowOffset,
    #[error("invalid size")]
    InvalidSize,
    #[error("invalid index")]
    InvalidIndex,
    #[error("invalid boundary constraint")]
    InvalidBoundaryConstraint,
    #[error("invalid integrity constraint")]
    InvalidIntegrityConstraint,
}
