use super::{
    Constant, ConstrainedBoundary, ConstraintDomain, IndexedTraceAccess, MatrixAccess,
    NamedTraceAccess, SymbolType, TraceSegment, VectorAccess, MIN_CYCLE_LENGTH,
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
    // --- INVALID ACCESS ERRORS ------------------------------------------------------------------

    pub(crate) fn invalid_vector_access(access: &VectorAccess, symbol_type: &SymbolType) -> Self {
        Self::InvalidUsage(format!(
            "Vector Access {}[{}] was declared as a {} which is not a supported type.",
            access.name(),
            access.idx(),
            symbol_type
        ))
    }

    pub(crate) fn invalid_matrix_access(access: &MatrixAccess, symbol_type: &SymbolType) -> Self {
        SemanticError::InvalidUsage(format!(
            "Matrix Access {}[{}][{}] was declared as a {} which is not a supported type.",
            access.name(),
            access.row_idx(),
            access.col_idx(),
            symbol_type
        ))
    }

    pub(crate) fn vector_access_out_of_bounds(access: &VectorAccess, vector_len: usize) -> Self {
        Self::IndexOutOfRange(format!(
            "Out-of-range index {} in vector {} of length {}",
            access.idx(),
            access.name(),
            vector_len
        ))
    }

    pub(crate) fn matrix_access_out_of_bounds(
        access: &MatrixAccess,
        matrix_row_len: usize,
        matrix_col_len: usize,
    ) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Out-of-range index [{}][{}] in matrix {} of dimensions ({}, {})",
            access.row_idx(),
            access.col_idx(),
            access.name(),
            matrix_row_len,
            matrix_col_len
        ))
    }

    pub(crate) fn named_trace_column_access_out_of_bounds(
        access: &NamedTraceAccess,
        size: usize,
    ) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Out-of-range index '{}' while accessing named trace column group '{}' of length {}",
            access.idx(),
            access.name(),
            size
        ))
    }

    pub(crate) fn trace_segment_access_out_of_bounds(trace_segment: usize, size: usize) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Trace segment index '{trace_segment}' is greater than the number of segments in the trace ({size}).",
        ))
    }

    pub(crate) fn indexed_trace_column_access_out_of_bounds(
        access: &IndexedTraceAccess,
        segment_width: u16,
    ) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Out-of-range index '{}' in trace segment '{}' of length {}",
            access.col_idx(),
            access.trace_segment(),
            segment_width
        ))
    }

    pub(crate) fn random_value_access_out_of_bounds(index: usize, size: u16) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Random value index {index} is greater than or equal to the total number of random values ({size})."
        ))
    }

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

    pub(crate) fn invalid_matrix_constant(constant: &Constant) -> Self {
        SemanticError::InvalidConstant(format!(
            "The matrix value of constant {} is invalid",
            constant.name()
        ))
    }

    // --- TYPE ERRORS ----------------------------------------------------------------------------

    pub(crate) fn unsupported_identifer_type(ident_name: &str, ident_type: &SymbolType) -> Self {
        SemanticError::InvalidUsage(format!(
            "Identifier {ident_name} was declared as a {ident_type} which is not a supported type."
        ))
    }

    pub(crate) fn not_a_trace_column_identifier(ident_name: &str, ident_type: &SymbolType) -> Self {
        SemanticError::InvalidUsage(format!(
            "Identifier {ident_name} was declared as a {ident_type} not as a trace column"
        ))
    }

    pub(crate) fn invalid_trace_binding(ident: &str) -> SemanticError {
        SemanticError::InvalidUsage(format!(
            "Expected {ident} to be a binding to a single trace column."
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

    pub(crate) fn trace_segment_mismatch(segment: TraceSegment) -> Self {
        SemanticError::InvalidUsage(format!(
            "The constraint expression cannot be enforced against trace segment {segment}"
        ))
    }

    pub(crate) fn invalid_list_folding(
        lf_value_type: &air_script_core::ListFoldingValueType,
        symbol_type: &SymbolType,
    ) -> SemanticError {
        SemanticError::InvalidListFolding(format!(
            "Symbol type {symbol_type} is not supported for list folding value type {lf_value_type:?}",
        ))
    }

    pub(crate) fn list_folding_empty_list(
        lf_value_type: &air_script_core::ListFoldingValueType,
    ) -> SemanticError {
        SemanticError::InvalidListFolding(format!(
            "List folding value cannot be an empty list. {lf_value_type:?} represents an empty list.",
        ))
    }
}
