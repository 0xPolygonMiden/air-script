use crate::error::SemanticError;

/// Struct to help with validation of AIR source.
pub(super) struct SourceValidator {
    trace_columns_exists: bool,
    public_inputs_exists: bool,
    boundary_constraints_exists: bool,
    transition_constraints_exists: bool,
}

impl SourceValidator {
    pub fn new() -> Self {
        SourceValidator {
            trace_columns_exists: false,
            public_inputs_exists: false,
            boundary_constraints_exists: false,
            transition_constraints_exists: false,
        }
    }

    /// If the declaration exists, sets corresponding boolean flag to true.
    pub fn exists(&mut self, section: &str) {
        match section {
            "trace_columns" => self.trace_columns_exists = true,
            "public_inputs" => self.public_inputs_exists = true,
            "boundary_constraints" => self.boundary_constraints_exists = true,
            "transition_constraints" => self.transition_constraints_exists = true,
            _ => unreachable!(),
        }
    }

    /// Returns a SemanticError if any of the required declarations are missing.
    pub fn check(&self) -> Result<(), SemanticError> {
        // make sure trace_columns are declared.
        if !self.trace_columns_exists {
            return Err(SemanticError::MissingDeclaration(
                "trace_columns section is missing".to_string(),
            ));
        }
        // make sure public_inputs are declared.
        if !self.public_inputs_exists {
            return Err(SemanticError::MissingDeclaration(
                "public_inputs section is missing".to_string(),
            ));
        }
        // make sure boundary_constraints are declared.
        if !self.boundary_constraints_exists {
            return Err(SemanticError::MissingDeclaration(
                "boundary_constraints section is missing".to_string(),
            ));
        }
        // make sure transition_constraints are declared.
        if !self.transition_constraints_exists {
            return Err(SemanticError::MissingDeclaration(
                "transition_constraints section is missing".to_string(),
            ));
        }

        Ok(())
    }
}
