use super::{
    AccessType, ConstantValueExpr, SemanticError, SymbolAccess, SymbolBinding, TraceAccess,
    TraceBinding, Value,
};

/// Symbol information for a constant, variable, trace column, periodic column, or public input.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Symbol {
    name: String,
    binding: SymbolBinding,
}

impl Symbol {
    pub(super) fn new(name: String, binding: SymbolBinding) -> Self {
        Self { name, binding }
    }

    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn binding(&self) -> &SymbolBinding {
        &self.binding
    }

    pub fn get_value(&self, symbol_access: SymbolAccess) -> Result<Value, SemanticError> {
        match self.binding() {
            SymbolBinding::Constant(constant_type) => {
                self.get_constant_value(constant_type, symbol_access)
            }
            SymbolBinding::PeriodicColumn(index, cycle_len) => {
                self.get_periodic_column_value(*index, *cycle_len, symbol_access)
            }
            SymbolBinding::PublicInput(size) => self.get_public_input_value(*size, symbol_access),
            SymbolBinding::RandomValues(offset, size) => {
                self.get_random_value(*offset, *size, symbol_access)
            }
            SymbolBinding::Trace(columns) => self.get_trace_value(columns, symbol_access),
            SymbolBinding::Variable(_) => {
                unreachable!("Variable values cannot be accessed directly, since they reference expressions which must be added to the graph");
            }
        }
    }

    // --- VALUE ACCESS HELPERS -------------------------------------------------------------------

    fn get_constant_value(
        &self,
        constant_type: &ConstantValueExpr,
        symbol_access: SymbolAccess,
    ) -> Result<Value, SemanticError> {
        if symbol_access.offset() != 0 {
            return Err(SemanticError::invalid_constant_access_type(
                symbol_access.name(),
                symbol_access.access_type(),
            ));
        }
        match symbol_access.access_type() {
            AccessType::Default => return Ok(Value::BoundConstant(symbol_access)),
            AccessType::Slice(_) => {
                return Err(SemanticError::invalid_constant_access_type(
                    symbol_access.name(),
                    symbol_access.access_type(),
                ));
            }
            AccessType::Vector(idx) => match constant_type {
                ConstantValueExpr::Scalar(_) => {
                    return Err(SemanticError::invalid_constant_access_type(
                        symbol_access.name(),
                        symbol_access.access_type(),
                    ))
                }
                ConstantValueExpr::Vector(vector) => {
                    if *idx >= vector.len() {
                        return Err(SemanticError::vector_access_out_of_bounds(
                            symbol_access.name(),
                            *idx,
                            vector.len(),
                        ));
                    }
                }
                ConstantValueExpr::Matrix(matrix) => {
                    if *idx >= matrix.len() {
                        return Err(SemanticError::vector_access_out_of_bounds(
                            symbol_access.name(),
                            *idx,
                            matrix.len(),
                        ));
                    }
                }
            },
            AccessType::Matrix(row_idx, col_idx) => match constant_type {
                ConstantValueExpr::Scalar(_) | ConstantValueExpr::Vector(_) => {
                    return Err(SemanticError::invalid_constant_access_type(
                        symbol_access.name(),
                        symbol_access.access_type(),
                    ))
                }
                ConstantValueExpr::Matrix(matrix) => {
                    if *row_idx >= matrix.len() || *col_idx >= matrix[0].len() {
                        return Err(SemanticError::matrix_access_out_of_bounds(
                            symbol_access.name(),
                            *row_idx,
                            *col_idx,
                            matrix.len(),
                            matrix[0].len(),
                        ));
                    }
                }
            },
        }

        Ok(Value::BoundConstant(symbol_access))
    }

    fn get_periodic_column_value(
        &self,
        index: usize,
        cycle_len: usize,
        symbol_access: SymbolAccess,
    ) -> Result<Value, SemanticError> {
        let (_, access_type, offset) = symbol_access.into_parts();
        match (access_type, offset) {
            (AccessType::Default, 0) => Ok(Value::PeriodicColumn(index, cycle_len)),
            _ => Err(SemanticError::invalid_periodic_column_access_type(
                self.name(),
            )),
        }
    }

    fn get_public_input_value(
        &self,
        size: usize,
        symbol_access: SymbolAccess,
    ) -> Result<Value, SemanticError> {
        let (_, access_type, offset) = symbol_access.into_parts();
        match (access_type, offset) {
            (AccessType::Vector(index), 0) => {
                if index >= size {
                    return Err(SemanticError::vector_access_out_of_bounds(
                        self.name(),
                        index,
                        size,
                    ));
                }
                return Ok(Value::PublicInput(self.name().to_string(), index));
            }
            _ => return Err(SemanticError::invalid_public_input_access_type(self.name())),
        }
    }

    fn get_random_value(
        &self,
        binding_offset: usize,
        binding_size: usize,
        symbol_access: SymbolAccess,
    ) -> Result<Value, SemanticError> {
        match (symbol_access.access_type(), symbol_access.offset()) {
            (AccessType::Default, 0) => {
                if binding_size != 1 {
                    return Err(SemanticError::invalid_random_value_binding_access(
                        self.name(),
                    ));
                }
                Ok(Value::RandomValue(binding_offset))
            }
            (AccessType::Vector(idx), 0) => {
                if *idx >= binding_size {
                    return Err(SemanticError::vector_access_out_of_bounds(
                        self.name(),
                        *idx,
                        binding_size,
                    ));
                }

                let offset = binding_offset + idx;
                Ok(Value::RandomValue(offset))
            }
            _ => Err(SemanticError::invalid_random_value_access_type(
                self.name(),
                symbol_access.access_type(),
            )),
        }
    }

    fn get_trace_value(
        &self,
        binding: &TraceBinding,
        symbol_access: SymbolAccess,
    ) -> Result<Value, SemanticError> {
        let (_, access_type, offset) = symbol_access.into_parts();
        match access_type {
            AccessType::Default => {
                if binding.size() != 1 {
                    return Err(SemanticError::invalid_trace_binding_access(self.name()));
                }
                let trace_segment = binding.trace_segment();
                let trace_access =
                    TraceAccess::new(trace_segment, binding.offset(), binding.size(), offset);
                Ok(Value::TraceElement(trace_access))
            }
            AccessType::Vector(idx) => {
                if idx >= binding.size() {
                    return Err(SemanticError::vector_access_out_of_bounds(
                        self.name(),
                        idx,
                        binding.size(),
                    ));
                }

                let trace_segment = binding.trace_segment();
                let trace_access =
                    TraceAccess::new(trace_segment, binding.offset() + idx, 1, offset);
                Ok(Value::TraceElement(trace_access))
            }
            _ => Err(SemanticError::invalid_trace_access_type(
                self.name(),
                &access_type,
            )),
        }
    }
}
