use super::{
    AccessType, ConstantType, ConstantValue, Identifier, IndexedTraceAccess, MatrixAccess,
    SemanticError, TraceColumns, TraceSegment, Value, VariableType, VectorAccess, AUX_SEGMENT,
    CURRENT_ROW, DEFAULT_SEGMENT,
};
use crate::constraints::ConstraintDomain;
use std::fmt::Display;

/// Symbol information for a constant, variable, trace column, periodic column, or public input.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Symbol {
    name: String,
    scope: Scope,
    symbol_type: SymbolType,
}

impl Symbol {
    pub(super) fn new(name: String, scope: Scope, symbol_type: SymbolType) -> Self {
        Self {
            name,
            scope,
            symbol_type,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn symbol_type(&self) -> &SymbolType {
        &self.symbol_type
    }

    fn validate_scope(&self, usage_scope: Scope) -> Result<(), SemanticError> {
        if self.scope != Scope::Global && self.scope != usage_scope {
            return Err(SemanticError::OutOfScope(format!(
                "Symbol {} is not available in {}",
                self.name(),
                usage_scope
            )));
        }

        Ok(())
    }

    // TODO: docs
    pub fn trace_segment(&self) -> TraceSegment {
        match self.symbol_type() {
            // The constraint target for random values defaults to the second (auxiliary) trace
            // segment.
            // TODO: make this more general, so random values from further trace segments can be
            // used. This requires having a way to describe different sets of randomness in
            // the AirScript syntax.
            SymbolType::RandomValuesBinding(_, _) => AUX_SEGMENT,
            SymbolType::TraceColumns(columns) => columns.trace_segment(),
            _ => DEFAULT_SEGMENT,
        }
    }

    // TODO: maybe refactor this to simplify; maybe a trait that operates by SymbolType?
    pub fn validate_access(&self, access_type: &AccessType) -> Result<(), SemanticError> {
        match access_type {
            AccessType::Default => Ok(()),
            AccessType::Vector(idx) => {
                let vector_len = match self.symbol_type() {
                    SymbolType::Constant(ConstantType::Vector(vector)) => vector.len(),
                    SymbolType::PublicInput(size) => *size,
                    SymbolType::RandomValuesBinding(_, size) => *size,
                    SymbolType::TraceColumns(trace_columns) => trace_columns.size(),
                    SymbolType::Variable(variable) => {
                        match variable {
                            // TODO: scalar can be ok; check this symbol in the future
                            VariableType::Scalar(_) => return Ok(()),
                            VariableType::Vector(vector) => vector.len(),
                            _ => return Err(SemanticError::invalid_vector_access(self, *idx)),
                        }
                    }
                    _ => return Err(SemanticError::invalid_vector_access(self, *idx)),
                };

                if *idx >= vector_len {
                    // TODO: restore other error
                    // return Err(SemanticError::vector_access_out_of_bounds(symbol, *idx));
                    return Err(SemanticError::invalid_vector_access(self, *idx));
                }

                Ok(())
            }
            AccessType::Matrix(row_idx, col_idx) => {
                let (row_len, col_len) = match self.symbol_type() {
                    SymbolType::Constant(ConstantType::Matrix(matrix)) => {
                        (matrix.len(), matrix[0].len())
                    }
                    SymbolType::Variable(variable) => {
                        match variable {
                            // TODO: scalar & vector can be ok; check this symbol in the future
                            VariableType::Scalar(_) | VariableType::Vector(_) => return Ok(()),
                            VariableType::Matrix(matrix) => (matrix.len(), matrix[0].len()),
                            _ => {
                                return Err(SemanticError::invalid_matrix_access(
                                    self, *row_idx, *col_idx,
                                ))
                            }
                        }
                    }
                    _ => {
                        return Err(SemanticError::invalid_matrix_access(
                            self, *row_idx, *col_idx,
                        ))
                    }
                };

                if *row_idx >= row_len || *col_idx >= col_len {
                    // TODO: restore other error
                    // return Err(SemanticError::matrix_access_out_of_bounds(
                    //     self, row_len, col_len,
                    // ));
                    return Err(SemanticError::invalid_matrix_access(
                        self, *row_idx, *col_idx,
                    ));
                }

                Ok(())
            }
        }
    }

    // TODO: return value details or AccessDetails w/ Value and TraceSegment
    pub fn access_value(&self, access_type: AccessType) -> Result<Value, SemanticError> {
        self.validate_access(&access_type)?;

        match self.symbol_type() {
            SymbolType::Constant(_) => match access_type {
                // TODO: maybe simplify this
                AccessType::Default => Ok(Value::Constant(ConstantValue::Scalar(
                    self.name().to_string(),
                ))),
                AccessType::Vector(idx) => {
                    // TODO: maybe simplify this
                    let access_ref = VectorAccess::new(Identifier(self.name().to_string()), idx);
                    Ok(Value::Constant(ConstantValue::Vector(access_ref)))
                }
                AccessType::Matrix(row_idx, col_idx) => {
                    // TODO: maybe simplify this
                    let access_ref =
                        MatrixAccess::new(Identifier(self.name().to_string()), row_idx, col_idx);
                    Ok(Value::Constant(ConstantValue::Matrix(access_ref)))
                }
            },
            SymbolType::PeriodicColumn(index, cycle_len) => match access_type {
                AccessType::Default => Ok(Value::PeriodicColumn(*index, *cycle_len)),
                _ => Err(SemanticError::invalid_periodic_column_usage(self.name())),
            },
            SymbolType::PublicInput(_) => match access_type {
                AccessType::Vector(vector_idx) => {
                    Ok(Value::PublicInput(self.name().to_string(), vector_idx))
                }
                _ => Err(SemanticError::invalid_public_input_usage(self.name())),
            },
            SymbolType::RandomValuesBinding(offset, _) => match access_type {
                // TODO: return AUX_SEGMENT
                AccessType::Default => Ok(Value::RandomValue(*offset)),
                AccessType::Vector(idx) => {
                    let offset = offset + idx;
                    Ok(Value::RandomValue(offset))
                }
                _ => Err(SemanticError::invalid_random_value_usage(self.name())),
            },
            SymbolType::TraceColumns(columns) => match access_type {
                AccessType::Default => {
                    // TODO: this should be checked somewhere else
                    if columns.size() != 1 {
                        return Err(SemanticError::invalid_trace_binding(self.name()));
                    }
                    let trace_segment = columns.trace_segment();
                    let trace_access =
                        IndexedTraceAccess::new(trace_segment, columns.offset(), CURRENT_ROW);
                    Ok(Value::TraceElement(trace_access))
                }
                AccessType::Vector(idx) => {
                    let trace_segment = columns.trace_segment();
                    let trace_access =
                        IndexedTraceAccess::new(trace_segment, columns.offset() + idx, CURRENT_ROW);
                    Ok(Value::TraceElement(trace_access))
                }
                _ => Err(SemanticError::invalid_trace_access(self.name())),
            },
            SymbolType::Variable(variable_type) => {
                todo!()
            }
        }
    }
}

/// The scope where an associated element can be used.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum Scope {
    BoundaryConstraints,
    IntegrityConstraints,
    Global,
}

impl From<ConstraintDomain> for Scope {
    fn from(domain: ConstraintDomain) -> Self {
        match domain {
            ConstraintDomain::FirstRow | ConstraintDomain::LastRow => Self::BoundaryConstraints,
            ConstraintDomain::EveryRow | ConstraintDomain::EveryFrame(_) => {
                Self::IntegrityConstraints
            }
        }
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BoundaryConstraints => write!(f, "boundary constraints scope"),
            Self::IntegrityConstraints => write!(f, "integrity constraints scope"),
            Self::Global => write!(f, "global scope"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum SymbolType {
    /// an identifier for a constant, containing its type and value
    Constant(ConstantType),
    /// an identifier for a trace column, containing trace column information with its trace
    /// segment, its size and its offset.
    TraceColumns(TraceColumns),
    /// an identifier for a public input, containing the size of the public input array
    PublicInput(usize),
    /// an identifier for a periodic column, containing its index out of all periodic columns and
    /// its cycle length in that order.
    PeriodicColumn(usize, usize),
    /// an identifier for a variable, containing its scope (boundary or integrity), name, and value
    Variable(VariableType),
    /// an identifier for random value, containing its index in the random values array and its
    /// length if this value is an array. For non-array random values second parameter is always 1.
    RandomValuesBinding(usize, usize),
}

impl Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(_) => write!(f, "Constant"),
            Self::PublicInput(_) => write!(f, "PublicInput"),
            Self::PeriodicColumn(_, _) => write!(f, "PeriodicColumn"),
            Self::TraceColumns(columns) => {
                write!(f, "TraceColumns in segment {}", columns.trace_segment())
            }
            Self::Variable(_) => write!(f, "Variable"),
            Self::RandomValuesBinding(_, _) => write!(f, "RandomValuesBinding"),
        }
    }
}
