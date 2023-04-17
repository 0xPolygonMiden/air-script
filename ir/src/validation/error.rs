use super::{
    AccessType, ConstrainedBoundary, ConstraintDomain, SymbolType, TraceAccess, TraceBindingAccess,
    TraceSegment, MIN_CYCLE_LENGTH,
};

#[derive(Debug)]
pub enum SemanticError {
    DuplicateIdentifier(String),
    IndexOutOfRange(String),
    InvalidConstant(String),
    InvalidConstraint(String),
    InvalidConstraintDomain(String),
    InvalidIdentifier(String),
    InvalidListComprehension(String),
    InvalidListFolding(String),
    InvalidPeriodicColumn(String),
    InvalidTraceSegment(String),
    InvalidUsage(String),
    MissingDeclaration(String),
    OutOfScope(String),
    TooManyConstraints(String),
}

impl SemanticError {
    // --- DECLARATION ERRORS ---------------------------------------------------------------------

    fn missing_section_declaration(missing_section: &str) -> Self {
        SemanticError::MissingDeclaration(format!("{missing_section} section is missing"))
    }

    pub(crate) fn missing_trace_columns_declaration() -> Self {
        Self::missing_section_declaration("trace_declaration")
    }

    pub(crate) fn missing_public_inputs_declaration() -> Self {
        Self::missing_section_declaration("public_inputs")
    }

    pub(crate) fn missing_boundary_constraints_declaration() -> Self {
        Self::missing_section_declaration("boundary_constraints")
    }

    pub(crate) fn missing_integrity_constraints_declaration() -> Self {
        Self::missing_section_declaration("integrity_constraints")
    }

    pub(crate) fn has_random_values_but_missing_aux_trace_columns_declaration() -> Self {
        SemanticError::MissingDeclaration(
            "random_values section requires aux_trace_columns section, which is missing"
                .to_string(),
        )
    }

    // --- ILLEGAL IDENTIFIER ERRORS --------------------------------------------------------------

    pub(crate) fn duplicate_identifer(
        ident_name: &str,
        ident_type: &SymbolType,
        prev_type: &SymbolType,
    ) -> Self {
        SemanticError::DuplicateIdentifier(format!(
            "Cannot declare {ident_name} as a {ident_type}, since it was already defined as a {prev_type}"))
    }

    pub(crate) fn undeclared_identifier(ident_name: &str) -> Self {
        SemanticError::InvalidIdentifier(format!("Identifier {ident_name} was not declared"))
    }

    // --- ILLEGAL VALUE ERRORS -------------------------------------------------------------------

    pub(crate) fn periodic_cycle_length_not_power_of_two(length: usize, cycle_name: &str) -> Self {
        SemanticError::InvalidPeriodicColumn(format!(
            "cycle length must be a power of two, but was {length} for cycle {cycle_name}"
        ))
    }

    pub(crate) fn periodic_cycle_length_too_small(length: usize, cycle_name: &str) -> Self {
        SemanticError::InvalidPeriodicColumn(format!(
            "cycle length must be at least {MIN_CYCLE_LENGTH}, but was {length} for cycle {cycle_name}"
        ))
    }

    pub(crate) fn invalid_matrix_constant(name: &str) -> Self {
        SemanticError::InvalidConstant(format!("The matrix value of constant {name} is invalid"))
    }

    // --- TYPE ERRORS ----------------------------------------------------------------------------

    pub(crate) fn not_a_trace_column_identifier(ident_name: &str, ident_type: &SymbolType) -> Self {
        SemanticError::InvalidUsage(format!(
            "Identifier {ident_name} was declared as a {ident_type} not as a trace column"
        ))
    }

    // --- INVALID ACCESS ERRORS ------------------------------------------------------------------

    pub(crate) fn invalid_constant_access_type(name: &str, access_type: &AccessType) -> Self {
        Self::InvalidUsage(format!(
            "ConstantBinding '{name}' cannot be accessed by a {access_type}.",
        ))
    }

    pub(crate) fn invalid_periodic_column_access_type(name: &str) -> Self {
        Self::InvalidUsage(format!(
            "Attempted to access periodic column '{name}' by index, which is not allowed.",
        ))
    }

    pub(crate) fn invalid_public_input_access_type(name: &str) -> Self {
        Self::InvalidUsage(format!(
            "Public input '{name}' can only be accessed by indexing into the vector.",
        ))
    }

    pub(crate) fn invalid_random_value_access_type(name: &str, access_type: &AccessType) -> Self {
        Self::InvalidUsage(format!(
            "Random value '{name}' cannot be indexed as a {access_type}.",
        ))
    }

    pub(crate) fn invalid_trace_access_type(name: &str, access_type: &AccessType) -> Self {
        Self::InvalidUsage(format!(
            "Trace column binding '{name}' cannot be accessed as a {access_type}.",
        ))
    }

    pub(crate) fn invalid_variable_access_type(name: &str, access_type: &AccessType) -> Self {
        Self::InvalidUsage(format!(
            "VariableBinding '{name}' cannot be accessed as a {access_type}.",
        ))
    }

    pub(crate) fn invalid_periodic_column_access_in_bc() -> SemanticError {
        SemanticError::InvalidUsage(
            "Periodic columns cannot be used in boundary constraints.".to_string(),
        )
    }

    pub(crate) fn invalid_public_input_access_in_ic() -> SemanticError {
        SemanticError::InvalidUsage(
            "Public inputs cannot be used in integrity constraints.".to_string(),
        )
    }

    pub(crate) fn invalid_random_value_binding_access(ident: &str) -> SemanticError {
        SemanticError::InvalidUsage(format!(
            "Expected {ident} to be a binding to a single random value."
        ))
    }

    pub(crate) fn invalid_trace_binding_access(ident: &str) -> SemanticError {
        SemanticError::InvalidUsage(format!(
            "Expected {ident} to be a binding to a single trace column."
        ))
    }

    pub(crate) fn invalid_trace_offset_in_bc(trace_access: &TraceAccess) -> SemanticError {
        SemanticError::InvalidUsage(format!(
            "Attempted to access trace column {} in a boundary constraint with a non-zero row offset of {}.", trace_access.col_idx(), trace_access.row_offset()
        ))
    }

    pub(crate) fn vector_access_out_of_bounds(
        name: &str,
        access_idx: usize,
        vector_len: usize,
    ) -> Self {
        Self::IndexOutOfRange(format!(
            "Out-of-range index {access_idx} in vector {name} of length {vector_len}",
        ))
    }

    pub(crate) fn matrix_access_out_of_bounds(
        name: &str,
        access_row: usize,
        access_col: usize,
        matrix_row_len: usize,
        matrix_col_len: usize,
    ) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Out-of-range index [{access_row}][{access_col}] in matrix {name} of dimensions ({matrix_row_len}, {matrix_col_len})",
        ))
    }

    pub(crate) fn indexed_trace_column_access_out_of_bounds(
        access: &TraceAccess,
        segment_width: u16,
    ) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Out-of-range index '{}' in trace segment '{}' of length {}",
            access.col_idx(),
            access.trace_segment(),
            segment_width
        ))
    }

    pub(crate) fn named_trace_column_access_out_of_bounds(
        access: &TraceBindingAccess,
        size: usize,
    ) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Out-of-range index '{}' while accessing named trace column group '{}' of length {}",
            access.col_offset(),
            access.name(),
            size
        ))
    }

    pub(crate) fn trace_segment_access_out_of_bounds(trace_segment: usize, size: usize) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Trace segment index '{trace_segment}' is greater than the number of segments in the trace ({size}).",
        ))
    }

    // --- INVALID CONSTRAINT ERRORS --------------------------------------------------------------

    pub(crate) fn incompatible_constraint_domains(
        base: &ConstraintDomain,
        other: &ConstraintDomain,
    ) -> Self {
        SemanticError::InvalidConstraintDomain(format!(
            "The specified constraint domains {base:?} and {other:?} are not compatible"
        ))
    }

    pub(crate) fn boundary_already_constrained(boundary: &ConstrainedBoundary) -> Self {
        SemanticError::TooManyConstraints(format!("A constraint was already defined at {boundary}"))
    }

    pub(crate) fn invalid_list_folding(
        lf_value_type: &air_script_core::ListFoldingValueExpr,
        symbol_type: &SymbolType,
    ) -> SemanticError {
        SemanticError::InvalidListFolding(format!(
            "Symbol type {symbol_type} is not supported for list folding value type {lf_value_type:?}",
        ))
    }

    pub(crate) fn list_folding_empty_list(
        lf_value_type: &air_script_core::ListFoldingValueExpr,
    ) -> SemanticError {
        SemanticError::InvalidListFolding(format!(
            "List folding value cannot be an empty list. {lf_value_type:?} represents an empty list.",
        ))
    }

    pub(crate) fn trace_segment_mismatch(segment: TraceSegment) -> Self {
        SemanticError::InvalidUsage(format!(
            "The constraint expression cannot be enforced against trace segment {segment}"
        ))
    }
}
