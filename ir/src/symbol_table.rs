use super::{
    trace_columns::TraceColumns, BTreeMap, Constant, ConstantType, Constants, Identifier,
    IndexedTraceAccess, MatrixAccess, NamedTraceAccess, PeriodicColumns, PublicInputs,
    SemanticError, TraceSegment, Variable, VariableType, VectorAccess, MIN_CYCLE_LENGTH,
};
use parser::ast::{PeriodicColumn, PublicInput, RandomValues, TraceCols};
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum IdentifierType {
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
    Variable(Scope, Variable),
    /// an identifier for random value, containing its index in the random values array and its
    /// length if this value is an array. For non-array random values second parameter is always 1.
    RandomValuesBinding(usize, usize),
}

impl Display for IdentifierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(_) => write!(f, "Constant"),
            Self::PublicInput(_) => write!(f, "PublicInput"),
            Self::PeriodicColumn(_, _) => write!(f, "PeriodicColumn"),
            Self::TraceColumns(columns) => {
                write!(f, "TraceColumns in segment {}", columns.trace_segment())
            }
            Self::Variable(Scope::BoundaryConstraints, _) => write!(f, "BoundaryVariable"),
            Self::Variable(Scope::IntegrityConstraints, _) => write!(f, "IntegrityVariable"),
            Self::RandomValuesBinding(_, _) => write!(f, "RandomValuesBinding"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum Scope {
    BoundaryConstraints,
    IntegrityConstraints,
}

/// SymbolTable for identifiers to track their types and information and enforce uniqueness of
/// identifiers.
#[derive(Default, Debug)]
pub struct SymbolTable {
    /// Vector in which index is trace segment and value is number of columns in this segment
    segment_widths: Vec<u16>,

    /// Number of random values. For array initialized in `rand: [n]` form it will be `n`, and for
    /// `rand: [a, b[n], c, ...]` it will be length of the flattened array.
    num_random_values: u16,

    /// A map of all declared identifiers from their name (the key) to their type.
    identifiers: BTreeMap<String, IdentifierType>,

    /// A vector of constants declared in the AirScript module.
    constants: Constants,

    /// A map of the Air's periodic columns using the index of the column within the declared
    /// periodic columns as the key and the vector of periodic values as the value
    periodic_columns: PeriodicColumns,

    /// A vector of public inputs with each value as a tuple of input identifier and it's array
    /// size.
    public_inputs: PublicInputs,
}

impl SymbolTable {
    /// Adds a declared identifier to the symbol table using the identifier as the key and the
    /// type the identifier represents as the value.
    ///
    /// # Errors
    /// It returns an error if the identifier already existed in the table.
    fn insert_symbol(
        &mut self,
        ident_name: &str,
        ident_type: IdentifierType,
    ) -> Result<(), SemanticError> {
        let result = self
            .identifiers
            .insert(ident_name.to_owned(), ident_type.clone());
        match result {
            Some(prev_type) => Err(SemanticError::DuplicateIdentifier(format!(
                "Cannot declare {ident_name} as a {ident_type}, since it was already defined as a {prev_type}"
            ))),
            None => Ok(()),
        }
    }

    // --- PUBLIC MUTATORS ------------------------------------------------------------------------

    /// Add all constants by their identifiers and values.
    pub(super) fn insert_constant(&mut self, constant: &Constant) -> Result<(), SemanticError> {
        let Identifier(name) = &constant.name();
        validate_constant(constant)?;
        self.insert_symbol(name, IdentifierType::Constant(constant.value().clone()))?;
        self.constants.push(constant.clone());

        Ok(())
    }

    /// Add all trace columns in the specified trace segment by their identifiers, sizes and indices.
    pub(super) fn insert_trace_columns(
        &mut self,
        trace_segment: TraceSegment,
        trace: &[TraceCols],
    ) -> Result<(), SemanticError> {
        if trace_segment >= self.segment_widths.len() as u8 {
            self.segment_widths.resize(trace_segment as usize + 1, 0);
        }

        let mut col_idx = 0;
        for trace_cols in trace {
            let trace_columns =
                TraceColumns::new(trace_segment, col_idx, trace_cols.size() as usize);
            self.insert_symbol(
                trace_cols.name(),
                IdentifierType::TraceColumns(trace_columns),
            )?;
            col_idx += trace_cols.size() as usize;
        }
        self.segment_widths[trace_segment as usize] = col_idx as u16;

        Ok(())
    }

    /// Adds all public inputs by their identifier names and array length.
    pub(super) fn insert_public_inputs(
        &mut self,
        public_inputs: &[PublicInput],
    ) -> Result<(), SemanticError> {
        for input in public_inputs.iter() {
            self.insert_symbol(input.name(), IdentifierType::PublicInput(input.size()))?;
            self.public_inputs
                .push((input.name().to_string(), input.size()));
        }

        Ok(())
    }

    /// Adds all periodic columns by their identifier names, their indices in the array of all
    /// periodic columns, and the lengths of their periodic cycles.
    pub(super) fn insert_periodic_columns(
        &mut self,
        columns: &[PeriodicColumn],
    ) -> Result<(), SemanticError> {
        for (index, column) in columns.iter().enumerate() {
            validate_cycles(column)?;
            let values = column.values().to_vec();

            self.insert_symbol(
                column.name(),
                IdentifierType::PeriodicColumn(index, values.len()),
            )?;
            self.periodic_columns.push(values);
        }

        Ok(())
    }

    /// Adds all random values by their identifier names and array length.
    pub(super) fn insert_random_values(
        &mut self,
        values: &RandomValues,
    ) -> Result<(), SemanticError> {
        self.num_random_values = values.size() as u16;
        let mut offset = 0;
        // add the name of the random values array to the symbol table
        self.insert_symbol(
            values.name(),
            IdentifierType::RandomValuesBinding(offset, values.size() as usize),
        )?;
        // add the named random value bindings to the symbol table
        for value in values.bindings() {
            self.insert_symbol(
                value.name(),
                IdentifierType::RandomValuesBinding(offset, value.size() as usize),
            )?;
            offset += value.size() as usize;
        }
        Ok(())
    }

    /// Inserts a boundary or integrity variable into the symbol table.
    pub(super) fn insert_variable(
        &mut self,
        scope: Scope,
        variable: &Variable,
    ) -> Result<(), SemanticError> {
        self.insert_symbol(
            variable.name(),
            IdentifierType::Variable(scope, variable.clone()),
        )?;
        Ok(())
    }

    /// Consumes this symbol table and returns the information required for declaring constants,
    /// public inputs, periodic columns and columns amount for the AIR.
    pub(super) fn into_declarations(self) -> (Vec<u16>, Constants, PublicInputs, PeriodicColumns) {
        (
            self.segment_widths,
            self.constants,
            self.public_inputs,
            self.periodic_columns,
        )
    }

    // --- ACCESSORS ------------------------------------------------------------------------------

    /// Gets the number of trace segments that were specified for this AIR.
    pub(super) fn num_trace_segments(&self) -> usize {
        self.segment_widths.len() + 1
    }

    /// Returns a vector containing the widths of all trace segments.
    pub(super) fn segment_widths(&self) -> &Vec<u16> {
        &self.segment_widths
    }

    /// Returns the number of random values that were specified for this AIR.
    pub(super) fn num_random_values(&self) -> u16 {
        self.num_random_values
    }

    /// Returns the type associated with the specified identifier name.
    ///
    /// # Errors
    /// Returns an error if the identifier was not in the symbol table.
    pub(super) fn get_type(&self, name: &str) -> Result<&IdentifierType, SemanticError> {
        if let Some(ident_type) = self.identifiers.get(name) {
            Ok(ident_type)
        } else {
            Err(SemanticError::InvalidIdentifier(format!(
                "Identifier {name} was not declared"
            )))
        }
    }

    /// Looks up a [NamedTraceAccess] by its identifier name and returns an equivalent
    /// [IndexedTraceAccess].
    ///
    /// # Errors
    /// Returns an error if:
    /// - the identifier was not in the symbol table.
    /// - the identifier was not declared as a trace column binding.
    pub(super) fn get_trace_access_by_name(
        &self,
        trace_access: &NamedTraceAccess,
    ) -> Result<IndexedTraceAccess, SemanticError> {
        let elem_type = self.get_type(trace_access.name())?;
        match elem_type {
            IdentifierType::TraceColumns(columns) => {
                if trace_access.idx() >= columns.size() {
                    return Err(SemanticError::IndexOutOfRange(format!(
                        "Out-of-range index '{}' while accessing named trace column group '{}' of length {}",
                        trace_access.idx(),
                        trace_access.name(),
                        columns.size()
                    )));
                }

                Ok(IndexedTraceAccess::new(
                    columns.trace_segment(),
                    columns.offset() + trace_access.idx(),
                    trace_access.row_offset(),
                ))
            }
            _ => Err(SemanticError::InvalidUsage(format!(
                "Identifier {} was declared as a {} not as a trace column",
                trace_access.name(),
                elem_type
            ))),
        }
    }

    /// Checks that the specified name and index are a valid reference to a declared public input
    /// or a vector constant and returns the symbol type. If it's not a valid reference, an error
    /// is returned.
    ///
    /// # Errors
    /// - Returns an error if the identifier is not in the symbol table.
    /// - Returns an error if the identifier is not associated with a vector access type.
    /// - Returns an error if the index is not in the declared public input array.
    /// - Returns an error if the index is greater than the vector's length.
    pub(super) fn access_vector_element(
        &self,
        vector_access: &VectorAccess,
    ) -> Result<&IdentifierType, SemanticError> {
        let symbol_type = self.get_type(vector_access.name())?;
        match symbol_type {
            IdentifierType::PublicInput(size) => {
                if vector_access.idx() < *size {
                    Ok(symbol_type)
                } else {
                    Err(SemanticError::public_inputs_out_of_bounds(
                        vector_access,
                        *size,
                    ))
                }
            }
            IdentifierType::Constant(ConstantType::Vector(vector)) => {
                validate_vector_access(vector_access, vector.len())?;
                Ok(symbol_type)
            }
            IdentifierType::Variable(_, variable) => match variable.value() {
                VariableType::Scalar(_) => Ok(symbol_type),
                VariableType::Vector(vector) => {
                    validate_vector_access(vector_access, vector.len())?;
                    Ok(symbol_type)
                }
                _ => Err(SemanticError::invalid_vector_access(
                    vector_access,
                    symbol_type,
                )),
            },
            IdentifierType::TraceColumns(trace_columns) => {
                if vector_access.idx() < trace_columns.size() {
                    Ok(symbol_type)
                } else {
                    Err(SemanticError::vector_access_out_of_bounds(
                        vector_access,
                        trace_columns.size(),
                    ))
                }
            }
            IdentifierType::RandomValuesBinding(_, size) => {
                if vector_access.idx() < *size {
                    Ok(symbol_type)
                } else {
                    Err(SemanticError::vector_access_out_of_bounds(
                        vector_access,
                        *size,
                    ))
                }
            }
            _ => Err(SemanticError::invalid_vector_access(
                vector_access,
                symbol_type,
            )),
        }
    }

    /// Checks that the specified name and index are a valid reference to a matrix constant and
    /// returns the symbol type. If it's not a valid reference, an error is returned.
    ///
    /// # Errors
    /// - Returns an error if the identifier is not in the symbol table.
    /// - Returns an error if the identifier is not associated with a matrix access type.
    /// - Returns an error if the row index is greater than the matrix row length.
    /// - Returns an error if the column index is greater than the matrix column length.
    pub(super) fn access_matrix_element(
        &self,
        matrix_access: &MatrixAccess,
    ) -> Result<&IdentifierType, SemanticError> {
        let symbol_type = self.get_type(matrix_access.name())?;
        match symbol_type {
            IdentifierType::Constant(ConstantType::Matrix(matrix)) => {
                validate_matrix_access(matrix_access, matrix.len(), matrix[0].len())?;
                Ok(symbol_type)
            }
            IdentifierType::Variable(_, variable) => match variable.value() {
                VariableType::Scalar(_) => Ok(symbol_type),
                VariableType::Vector(_) => Ok(symbol_type),
                VariableType::Matrix(matrix) => {
                    validate_matrix_access(matrix_access, matrix.len(), matrix[0].len())?;
                    Ok(symbol_type)
                }
                _ => Err(SemanticError::invalid_matrix_access(
                    matrix_access,
                    symbol_type,
                )),
            },
            _ => Err(SemanticError::invalid_matrix_access(
                matrix_access,
                symbol_type,
            )),
        }
    }

    /// Checks that the specified trace access is valid, i.e. that it references a declared trace
    /// segment and the index is within the bounds of the declared segment width.
    ///
    /// # Errors
    /// Returns an error if:
    /// - the specified trace segment is out of range.
    /// - the specified column index is out of range.
    pub(super) fn validate_trace_access(
        &self,
        trace_access: &IndexedTraceAccess,
    ) -> Result<(), SemanticError> {
        let segment_idx = trace_access.trace_segment() as usize;
        if segment_idx > self.segment_widths().len() {
            return Err(SemanticError::IndexOutOfRange(format!(
                "Segment index '{}' is greater than the number of segments in the trace ({}).",
                segment_idx,
                self.segment_widths().len()
            )));
        }
        if trace_access.col_idx() as u16 >= self.segment_widths()[segment_idx] {
            return Err(SemanticError::IndexOutOfRange(format!(
                "Out-of-range index '{}' in trace segment '{}' of length {}",
                trace_access.col_idx(),
                trace_access.trace_segment(),
                self.segment_widths()[segment_idx]
            )));
        }

        Ok(())
    }

    /// Checks that the specified random value access index is valid, i.e. that it is in range of
    /// the number of declared random values.
    pub(super) fn validate_rand_access(&self, index: usize) -> Result<(), SemanticError> {
        if index >= usize::from(self.num_random_values()) {
            return Err(SemanticError::IndexOutOfRange(format!(
                "Random value index {} is greater than or equal to the total number of random values ({}).", 
                index,
                self.num_random_values()
            )));
        }

        Ok(())
    }
}

// HELPERS
// ================================================================================================

/// Validates the cycle length of the specified periodic column.
fn validate_cycles(column: &PeriodicColumn) -> Result<(), SemanticError> {
    let name = column.name();
    let cycle = column.values().len();

    if !cycle.is_power_of_two() {
        return Err(SemanticError::InvalidPeriodicColumn(format!(
            "cycle length must be a power of two, but was {cycle} for cycle {name}"
        )));
    }

    if cycle < MIN_CYCLE_LENGTH {
        return Err(SemanticError::InvalidPeriodicColumn(format!(
            "cycle length must be at least {MIN_CYCLE_LENGTH}, but was {cycle} for cycle {name}"
        )));
    }

    Ok(())
}

/// Checks that the declared value of a constant is valid.
fn validate_constant(constant: &Constant) -> Result<(), SemanticError> {
    match constant.value() {
        // check the number of elements in each row are same for a matrix
        ConstantType::Matrix(matrix) => {
            let row_len = matrix[0].len();
            if matrix.iter().skip(1).all(|row| row.len() == row_len) {
                Ok(())
            } else {
                Err(SemanticError::InvalidConstant(format!(
                    "The matrix value of constant {} is invalid",
                    constant.name()
                )))
            }
        }
        _ => Ok(()),
    }
}

/// Checks that the specified vector access index is valid and returns an error otherwise.
fn validate_vector_access(
    vector_access: &VectorAccess,
    vector_len: usize,
) -> Result<(), SemanticError> {
    if vector_access.idx() >= vector_len {
        return Err(SemanticError::vector_access_out_of_bounds(
            vector_access,
            vector_len,
        ));
    }
    Ok(())
}

/// Checks that the specified matrix access indices are valid and returns an error otherwise.
fn validate_matrix_access(
    matrix_access: &MatrixAccess,
    matrix_row_len: usize,
    matrix_col_len: usize,
) -> Result<(), SemanticError> {
    if matrix_access.row_idx() >= matrix_row_len {
        return Err(SemanticError::matrix_access_out_of_bounds(
            matrix_access,
            matrix_row_len,
            matrix_col_len,
        ));
    }
    if matrix_access.col_idx() >= matrix_col_len {
        return Err(SemanticError::matrix_access_out_of_bounds(
            matrix_access,
            matrix_row_len,
            matrix_col_len,
        ));
    }
    Ok(())
}
