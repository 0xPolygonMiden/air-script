use super::{NamedTraceAccess, SemanticError, Symbol, SymbolType};

/// TODO: docs
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum AccessType {
    Default,
    Vector(usize),
    /// TODO: docs (row, then column)
    Matrix(usize, usize),
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

impl ValidateIdentifierAccess for NamedTraceAccess {
    fn validate(&self, symbol: &Symbol) -> Result<(), SemanticError> {
        match symbol.symbol_type() {
            SymbolType::TraceColumns(columns) => {
                if self.idx() >= columns.size() {
                    return Err(SemanticError::named_trace_column_access_out_of_bounds(
                        self,
                        columns.size(),
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
