use crate::error::SemanticError;

/// Struct to store existence of sections in the AIR source code
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

    /// If the section exists, sets the provided section boolean to true.
    pub fn exists(&mut self, section: &str) {
        match section {
            "trace_columns" => self.trace_columns_exists = true,
            "public_inputs" => self.public_inputs_exists = true,
            "boundary_constraints" => self.boundary_constraints_exists = true,
            "transition_constraints" => self.transition_constraints_exists = true,
            _ => unreachable!(),
        }
    }

    /// Returns a MissingSourceSection error if the provided section is missing.
    pub fn check(&self, section: &str) -> Result<(), SemanticError> {
        match section {
            "trace_columns" => {
                if !self.trace_columns_exists {
                    return Err(SemanticError::MissingSourceSection(
                        "trace_columns section is missing".to_string(),
                    ));
                }
            }
            "public_inputs" => {
                if !self.public_inputs_exists {
                    return Err(SemanticError::MissingSourceSection(
                        "public_inputs section is missing".to_string(),
                    ));
                }
            }
            "boundary_constraints" => {
                if !self.boundary_constraints_exists {
                    return Err(SemanticError::MissingSourceSection(
                        "boundary_constraints section is missing".to_string(),
                    ));
                }
            }
            "transition_constraints" => {
                if !self.transition_constraints_exists {
                    return Err(SemanticError::MissingSourceSection(
                        "transition_constraints section is missing".to_string(),
                    ));
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}
