use super::{ConstantType, SemanticError, Symbol, SymbolType, TraceBindingAccess, VariableType};
use std::fmt::Display;

/// TODO: docs
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum AccessType {
    Default,
    Vector(usize),
    /// TODO: docs (row, then column)
    Matrix(usize, usize),
}

impl Display for AccessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "direct reference by name"),
            Self::Vector(_) => write!(f, "vector"),
            Self::Matrix(_, _) => write!(f, "matrix"),
        }
    }
}

/// Checks that the specified access into an identifier is valid and returns an error otherwise.
/// # Errors:
/// - Returns an error if the type of the identifier does not allow the access type. For example,
///   VariableType::Vector does not allow a MatrixAccess.
/// - Returns an error if any indices specified for the access are out of bounds fo the specified
///   identifier.
pub(super) trait ValidateIdentifierAccess {
    fn validate(&self, symbol: &Symbol) -> Result<(), SemanticError>;
}

impl ValidateIdentifierAccess for TraceBindingAccess {
    fn validate(&self, symbol: &Symbol) -> Result<(), SemanticError> {
        match symbol.symbol_type() {
            SymbolType::TraceColumns(trace_binding) | SymbolType::Parameter(trace_binding) => {
                if self.col_offset() >= trace_binding.size() {
                    return Err(SemanticError::named_trace_column_access_out_of_bounds(
                        self,
                        trace_binding.size(),
                    ));
                }
            }
            _ => {
                return Err(SemanticError::not_a_trace_column_identifier(
                    symbol.name(),
                    symbol.symbol_type(),
                ))
            }
        }

        Ok(())
    }
}

pub(crate) trait ValidateAccess {
    fn validate(&self, name: &str, access_type: &AccessType) -> Result<(), SemanticError>;
}

impl ValidateAccess for ConstantType {
    fn validate(&self, name: &str, access_type: &AccessType) -> Result<(), SemanticError> {
        match access_type {
            AccessType::Default => return Ok(()),
            AccessType::Vector(idx) => match self {
                ConstantType::Scalar(_) => {
                    return Err(SemanticError::invalid_constant_access_type(
                        name,
                        access_type,
                    ))
                }
                ConstantType::Vector(vector) => {
                    if *idx >= vector.len() {
                        return Err(SemanticError::vector_access_out_of_bounds(
                            name,
                            *idx,
                            vector.len(),
                        ));
                    }
                }
                ConstantType::Matrix(matrix) => {
                    if *idx >= matrix.len() {
                        return Err(SemanticError::vector_access_out_of_bounds(
                            name,
                            *idx,
                            matrix.len(),
                        ));
                    }
                }
            },
            AccessType::Matrix(row_idx, col_idx) => match self {
                ConstantType::Scalar(_) | ConstantType::Vector(_) => {
                    return Err(SemanticError::invalid_constant_access_type(
                        name,
                        access_type,
                    ))
                }
                ConstantType::Matrix(matrix) => {
                    if *row_idx >= matrix.len() || *col_idx >= matrix[0].len() {
                        return Err(SemanticError::matrix_access_out_of_bounds(
                            name,
                            *row_idx,
                            *col_idx,
                            matrix.len(),
                            matrix[0].len(),
                        ));
                    }
                }
            },
        }

        Ok(())
    }
}

impl ValidateAccess for VariableType {
    fn validate(&self, name: &str, access_type: &AccessType) -> Result<(), SemanticError> {
        match access_type {
            AccessType::Default => return Ok(()),
            AccessType::Vector(idx) => match self {
                // TODO: scalar can be ok; check this symbol in the future
                VariableType::Scalar(_) => return Ok(()),
                VariableType::Vector(vector) => {
                    if *idx >= vector.len() {
                        return Err(SemanticError::vector_access_out_of_bounds(
                            name,
                            *idx,
                            vector.len(),
                        ));
                    }
                }
                _ => {
                    return Err(SemanticError::invalid_variable_access_type(
                        name,
                        access_type,
                    ))
                }
            },
            AccessType::Matrix(row_idx, col_idx) => match self {
                // TODO: scalar & vector can be ok; check this symbol in the future
                VariableType::Scalar(_) | VariableType::Vector(_) => return Ok(()),
                VariableType::Matrix(matrix) => {
                    if *row_idx >= matrix.len() || *col_idx >= matrix[0].len() {
                        return Err(SemanticError::matrix_access_out_of_bounds(
                            name,
                            *row_idx,
                            *col_idx,
                            matrix.len(),
                            matrix[0].len(),
                        ));
                    }
                }
                _ => {
                    return Err(SemanticError::invalid_variable_access_type(
                        name,
                        access_type,
                    ))
                }
            },
        }

        Ok(())
    }
}
