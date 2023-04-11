use super::{AccessType, ConstantValueExpr, SemanticError, Symbol, SymbolType, TraceBindingAccess};

/// Checks that the specified access into an identifier is valid and returns an error otherwise.
/// # Errors:
/// - Returns an error if the type of the identifier does not allow the access type. For example,
///   VariableValueExpr::Vector does not allow a MatrixAccess.
/// - Returns an error if any indices specified for the access are out of bounds fo the specified
///   identifier.
pub(super) trait ValidateIdentifierAccess {
    fn validate(&self, symbol: &Symbol) -> Result<(), SemanticError>;
}

impl ValidateIdentifierAccess for TraceBindingAccess {
    fn validate(&self, symbol: &Symbol) -> Result<(), SemanticError> {
        match symbol.symbol_type() {
            SymbolType::TraceBinding(trace_binding) => {
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

impl ValidateAccess for ConstantValueExpr {
    fn validate(&self, name: &str, access_type: &AccessType) -> Result<(), SemanticError> {
        match access_type {
            AccessType::Default => return Ok(()),
            AccessType::Vector(idx) => match self {
                ConstantValueExpr::Scalar(_) => {
                    return Err(SemanticError::invalid_constant_access_type(
                        name,
                        access_type,
                    ))
                }
                ConstantValueExpr::Vector(vector) => {
                    if *idx >= vector.len() {
                        return Err(SemanticError::vector_access_out_of_bounds(
                            name,
                            *idx,
                            vector.len(),
                        ));
                    }
                }
                ConstantValueExpr::Matrix(matrix) => {
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
                ConstantValueExpr::Scalar(_) | ConstantValueExpr::Vector(_) => {
                    return Err(SemanticError::invalid_constant_access_type(
                        name,
                        access_type,
                    ))
                }
                ConstantValueExpr::Matrix(matrix) => {
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
