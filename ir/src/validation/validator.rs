use super::SemanticError;

/// Struct to help with validation of AIR source.
pub(crate) struct SourceValidator {
    main_trace_columns_exists: bool,
    aux_trace_columns_exists: bool,
    public_inputs_exists: bool,
    boundary_constraints_exists: bool,
    integrity_constraints_exists: bool,
    random_values_exists: bool,
}

impl SourceValidator {
    pub fn new() -> Self {
        SourceValidator {
            main_trace_columns_exists: false,
            aux_trace_columns_exists: false,
            public_inputs_exists: false,
            boundary_constraints_exists: false,
            integrity_constraints_exists: false,
            random_values_exists: false,
        }
    }

    /// If the declaration exists, sets corresponding boolean flag to true.
    pub fn exists(&mut self, section: &str) {
        match section {
            "main_trace_columns" => self.main_trace_columns_exists = true,
            "aux_trace_columns" => self.aux_trace_columns_exists = true,
            "public_inputs" => self.public_inputs_exists = true,
            "boundary_constraints" => self.boundary_constraints_exists = true,
            "integrity_constraints" => self.integrity_constraints_exists = true,
            "random_values" => self.random_values_exists = true,
            _ => unreachable!(),
        }
    }

    /// Returns a SemanticError if any of the required declarations are missing.
    pub fn check(&self) -> Result<(), SemanticError> {
        // make sure trace_columns are declared.
        if !self.main_trace_columns_exists {
            return Err(SemanticError::missing_trace_columns_declaration());
        }
        // make sure public_inputs are declared.
        if !self.public_inputs_exists {
            return Err(SemanticError::missing_public_inputs_declaration());
        }
        // make sure boundary_constraints are declared.
        if !self.boundary_constraints_exists {
            return Err(SemanticError::missing_boundary_constraints_declaration());
        }
        // make sure integrity_constraints are declared.
        if !self.integrity_constraints_exists {
            return Err(SemanticError::missing_integrity_constraints_declaration());
        }
        // make sure random_values are declared only if aux trace columns are declared
        if !self.aux_trace_columns_exists && self.random_values_exists {
            return Err(
                SemanticError::has_random_values_but_missing_aux_trace_columns_declaration(),
            );
        }

        Ok(())
    }
}
