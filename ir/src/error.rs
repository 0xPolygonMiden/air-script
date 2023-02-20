use super::{MatrixAccess, VectorAccess, NamedTraceAccess, IndexedTraceAccess};

use crate::symbol_table::IdentifierType;

#[derive(Debug)]
pub enum SemanticError {
    DuplicateIdentifier(String),
    IndexOutOfRange(String),
    InvalidConstant(String),
    InvalidConstraint(String),
    InvalidConstraintDomain(String),
    InvalidIdentifier(String),
    InvalidPeriodicColumn(String),
    InvalidUsage(String),
    MissingDeclaration(String),
    TooManyConstraints(String),
}

impl SemanticError {

    // --- INVALID ACCESS ERRORS ------------------------------------------------------------------

    pub(super) fn invalid_vector_access(
        access: &VectorAccess,
        symbol_type: &IdentifierType,
    ) -> Self {
        Self::InvalidUsage(format!(
            "Vector Access {}[{}] was declared as a {} which is not a supported type.",
            access.name(),
            access.idx(),
            symbol_type
        ))
    }

    pub(super) fn invalid_matrix_access(
        access: &MatrixAccess,
        symbol_type: &IdentifierType,
    ) -> Self {
        SemanticError::InvalidUsage(format!(
            "Matrix Access {}[{}][{}] was declared as a {} which is not a supported type.",
            access.name(),
            access.row_idx(),
            access.col_idx(),
            symbol_type
        ))
    }

    pub(super) fn vector_access_out_of_bounds(
        access: &VectorAccess,
        vector_len: usize
    ) -> Self {
        Self::IndexOutOfRange(format!(
            "Out-of-range index {} in vector constant {} of length {}",
            access.idx(),
            access.name(),
            vector_len
        ))
    }

    pub(super) fn public_inputs_out_of_bounds(
        access: &VectorAccess,
        size: usize
    ) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Out-of-range index {} in public input {} of length {}",
            access.idx(),
            access.name(),
            size
        ))
    }

    pub(super) fn matrix_access_out_of_bounds(
        access: &MatrixAccess,
        matrix_row_len: usize,
        matrix_col_len: usize,
    ) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Out-of-range index [{}][{}] in matrix constant {} of dimensions ({}, {})",
            access.row_idx(),
            access.col_idx(),
            access.name(),
            matrix_row_len,
            matrix_col_len
        ))
    }

    pub(super) fn named_trace_column_access_out_of_bounds(
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

    pub(super) fn indexed_trace_access_out_of_bounds(
        access: &IndexedTraceAccess,
        size: usize
    ) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Segment index '{}' is greater than the number of segments in the trace ({}).",
            access.trace_segment(),
            size
        ))
    }

    pub(super) fn indexed_trace_column_access_out_of_bounds(
        access: &IndexedTraceAccess,
        segment_width: u16
    ) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Out-of-range index '{}' in trace segment '{}' of length {}",
            access.col_idx(),
            access.trace_segment(),
            segment_width
        ))
    }

    pub(super) fn random_value_access_out_of_bounds(
        index: usize,
        size: u16
    ) -> Self {
        SemanticError::IndexOutOfRange(format!(
            "Random value index {index} is greater than or equal to the total number of random values ({size})."
        ))
    }

    // --- MISSING DECLARATION ERRORS -------------------------------------------------------------

    fn missing_declaration(
        missing_section: &str
    ) -> Self {
        SemanticError::MissingDeclaration(format!(
            "{missing_section} section is missing"))
    }

    pub(super) fn missing_trace_columns_declaration(
    ) -> Self {
        return Self::missing_declaration("trace_declaration");
    }

    pub(super) fn missing_public_inputs_declaration(
    ) -> Self {
        return Self::missing_declaration("public_inputs");
    }

    pub(super) fn missing_boundary_constraints_declaration(
    ) -> Self {
        return Self::missing_declaration("boundary_constraints");
    }

    pub(super) fn missing_integrity_constraints_declaration(
    ) -> Self {
        return Self::missing_declaration("integrity_constraints");
    }

    pub(super) fn has_random_values_but_missing_aux_trace_columns_declaration(
    ) -> Self {
        SemanticError::MissingDeclaration(format!(
            "random_values section requires aux_trace_columns section, which is missing"
        ))
    }

    // --- ILLEGAL IDENTIFIER ERRORS --------------------------------------------------------------

    pub(super) fn duplicate_identifer(
        ident_name: &str,
        ident_type: IdentifierType,
        prev_type: IdentifierType
    ) -> Self {
        SemanticError::DuplicateIdentifier(format!(
            "Cannot declare {ident_name} as a {ident_type}, since it was already defined as a {prev_type}"))
    }

    pub(super) fn undeclared_identifier(
        ident_name: &str
    ) -> Self {
        SemanticError::InvalidIdentifier(format!(
            "Identifier {ident_name} was not declared"
        ))
    }

    pub(super) fn illegal_trace_column_identifier(
        ident_name: &str,
        ident_type: &IdentifierType
    ) -> Self {
        SemanticError::InvalidUsage(format!(
            "Identifier {ident_name} was declared as a {ident_type} not as a trace column"
        ))
    }
    
    // --- ILLEGAL VALUE ERRORS -------------------------------------------------------------------

    pub(super) fn cycle_length_not_power_of_two(
        length: usize,
        cycle_name: &str
    ) -> 
        
    
}
