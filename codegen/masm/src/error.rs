#[derive(Debug)]
pub enum CodegenError {
    DuplicatedConstant,
    InvalidAccessType,
    UnknownConstant,
    InvalidRowOffset,
    InvalidSize,
    InvalidIndex,
    InvalidBoundaryConstraint,
    InvalidIntegrityConstraint,
}
