#[derive(Debug)]
pub enum ConstraintEvaluationError {
    InvalidTraceSegment(String),
    InvalidOperation(String),
    IdentifierNotFound(String),
    ConstantNotFound(String),
    PublicInputNotFound(String),
    OperationNotFound(String),
    InvalidConstantType(String),
}

impl ConstraintEvaluationError {
    pub fn invalid_trace_segment(segment: u8) -> Self {
        ConstraintEvaluationError::InvalidTraceSegment(format!(
            "Trace segment {} is invalid",
            segment
        ))
    }

    pub fn identifier_not_found(name: &str) -> Self {
        ConstraintEvaluationError::IdentifierNotFound(format!(
            "Identifier {} not found in JSON arrays",
            name
        ))
    }

    pub fn constant_not_found(name: &str) -> Self {
        ConstraintEvaluationError::ConstantNotFound(format!("Constant \"{}\" not found", name))
    }

    pub fn public_input_not_found(name: &str) -> Self {
        ConstraintEvaluationError::PublicInputNotFound(format!(
            "Public Input \"{}\" not found",
            name
        ))
    }

    pub fn invalid_constant_type(name: &str, constant_type: &str) -> Self {
        ConstraintEvaluationError::InvalidConstantType(format!(
            "Invalid type of constant \"{}\". {} exprected.",
            name, constant_type
        ))
    }

    pub fn operation_not_found(index: usize) -> Self {
        ConstraintEvaluationError::OperationNotFound(format!(
            "Operation with index {} does not match the expression in the expressions JSON array",
            index
        ))
    }
}
