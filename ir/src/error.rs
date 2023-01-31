use super::{MatrixAccess, VectorAccess};
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

    pub(super) fn vector_access_out_of_bounds(access: &VectorAccess, vector_len: usize) -> Self {
        Self::IndexOutOfRange(format!(
            "Out-of-range index {} in vector constant {} of length {}",
            access.idx(),
            access.name(),
            vector_len
        ))
    }

    pub(super) fn public_inputs_out_of_bounds(access: &VectorAccess, size: usize) -> Self {
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
}
