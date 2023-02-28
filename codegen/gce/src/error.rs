#[derive(Debug)]
pub enum ConstraintEvaluationError {
    ConstantNotFound(String),
    InvalidConstantType(String),
    InvalidOperation(String),
    InvalidTraceSegment(String),
    OperationNotFound(String),
}

impl ConstraintEvaluationError {
    pub fn invalid_trace_segment(segment: u8) -> Self {
        ConstraintEvaluationError::InvalidTraceSegment(format!(
            "Trace segment {segment} is invalid."
        ))
    }

    pub fn constant_not_found(name: &str) -> Self {
        ConstraintEvaluationError::ConstantNotFound(format!("Constant \"{name}\" not found."))
    }

    pub fn invalid_constant_type(name: &str, constant_type: &str) -> Self {
        ConstraintEvaluationError::InvalidConstantType(format!(
            "Invalid type of constant \"{name}\". Expected \"{constant_type}\"."
        ))
    }

    pub fn operation_not_found(index: usize) -> Self {
        ConstraintEvaluationError::OperationNotFound(format!(
            "Operation with index {index} does not match the expression in the JSON expressions array."
        ))
    }
}
