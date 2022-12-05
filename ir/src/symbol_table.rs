use super::{
    BTreeMap, Constants, PeriodicColumns, PublicInputs, SemanticError, TraceSegment,
    MIN_CYCLE_LENGTH,
};
use parser::ast::{
    constants::{Constant, ConstantType},
    Identifier, PeriodicColumn, PublicInput,
};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub(super) enum IdentifierType {
    /// an identifier for a constant, containing it's type and value
    Constant(ConstantType),
    /// an identifier for a trace column, containing trace column information with its trace segment
    /// and the index of the column in that segment.
    TraceColumn(TraceColumn),
    /// an identifier for a public input, containing the size of the public input array
    PublicInput(usize),
    /// an identifier for a periodic column, containing its index out of all periodic columns and
    /// its cycle length in that order.
    PeriodicColumn(usize, usize),
}

impl Display for IdentifierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(_) => write!(f, "Constant"),
            Self::PublicInput(_) => write!(f, "PublicInput"),
            Self::PeriodicColumn(_, _) => write!(f, "PeriodicColumn"),
            Self::TraceColumn(column) => {
                write!(f, "TraceColumn in segment {}", column.trace_segment())
            }
        }
    }
}

/// Describes a column in the execution trace by the trace segment to which it belongs and its
/// index within that segment.
#[derive(Debug, Copy, Clone)]
pub struct TraceColumn {
    trace_segment: TraceSegment,
    col_idx: usize,
}

impl TraceColumn {
    /// Creates a [TraceColumn] in the specified trace segment at the specified index.
    fn new(trace_segment: TraceSegment, col_idx: usize) -> Self {
        Self {
            trace_segment,
            col_idx,
        }
    }

    /// Gets the trace segment of this [TraceColumn].
    pub fn trace_segment(&self) -> TraceSegment {
        self.trace_segment
    }

    /// Gets the column index of this [TraceColumn].
    pub fn col_idx(&self) -> usize {
        self.col_idx
    }
}

/// SymbolTable for identifiers to track their types and information and enforce uniqueness of
/// identifiers.
#[derive(Default, Debug)]
pub(super) struct SymbolTable {
    /// The number of trace segments in the AIR.
    num_trace_segments: usize,

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
    /// Sets the number of trace segments to the maximum of `num_trace_segments` and the provided
    /// trace segment identifier, which is indexed from zero.
    fn set_num_trace_segments(&mut self, trace_segment: TraceSegment) {
        self.num_trace_segments = self.num_trace_segments.max((trace_segment + 1).into())
    }

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
                "Cannot declare {} as a {}, since it was already defined as a {}",
                ident_name, ident_type, prev_type
            ))),
            None => Ok(()),
        }
    }

    // --- PUBLIC MUTATORS ------------------------------------------------------------------------

    /// Add all constants by their identifiers and values.
    pub(super) fn insert_constants(&mut self, constants: &[Constant]) -> Result<(), SemanticError> {
        for constant in constants {
            let Identifier(name) = &constant.name();
            validate_constant(constant)?;
            self.insert_symbol(name, IdentifierType::Constant(constant.value().clone()))?;
            self.constants.push(constant.clone());
        }

        Ok(())
    }

    /// Add all trace columns in the specified trace segment by their identifiers and indices.
    pub(super) fn insert_trace_columns(
        &mut self,
        trace_segment: TraceSegment,
        columns: &[Identifier],
    ) -> Result<(), SemanticError> {
        self.set_num_trace_segments(trace_segment);

        for (idx, Identifier(name)) in columns.iter().enumerate() {
            let trace_column = TraceColumn::new(trace_segment, idx);
            self.insert_symbol(name, IdentifierType::TraceColumn(trace_column))?;
        }

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

    /// Consumes this symbol table and returns the information required for declaring constants,
    /// public inputs and periodic columns for the AIR.
    pub(super) fn into_declarations(self) -> (Constants, PublicInputs, PeriodicColumns) {
        (self.constants, self.public_inputs, self.periodic_columns)
    }

    // --- ACCESSORS ------------------------------------------------------------------------------

    /// Gets the number of trace segments that were specified for this AIR.
    pub(super) fn num_trace_segments(&self) -> usize {
        self.num_trace_segments
    }

    /// Returns the type associated with the specified identifier name.
    ///
    /// # Errors
    /// Returns an error if the identifier was not in the symbol table.
    pub(super) fn get_type(&self, name: &str) -> Result<IdentifierType, SemanticError> {
        if let Some(ident_type) = self.identifiers.get(name) {
            Ok(ident_type.clone())
        } else {
            Err(SemanticError::InvalidIdentifier(format!(
                "Identifier {} was not declared",
                name
            )))
        }
    }

    /// Checks that the specified name and index are a valid reference to a declared public input
    /// or a vector constant. If not, it returns an error.
    ///
    /// # Errors
    /// - Returns an error if the identifier is not associated with a vector access type.
    /// - Returns an error if the index is not in the declared public input array.
    /// - Returns an error if the index is greater than the vector's length.
    pub(super) fn validate_vector_access(
        &self,
        name: &str,
        idx: usize,
    ) -> Result<(), SemanticError> {
        let vector_access_type = self.get_type(name)?;
        match vector_access_type {
            IdentifierType::PublicInput(size) => {
                if idx < size {
                    Ok(())
                } else {
                    Err(SemanticError::IndexOutOfRange(format!(
                        "Out-of-range index {} in public input {} of length {}",
                        idx, name, size
                    )))
                }
            }
            IdentifierType::Constant(ConstantType::Vector(vector)) => {
                if idx < vector.len() {
                    Ok(())
                } else {
                    Err(SemanticError::IndexOutOfRange(format!(
                        "Out-of-range index {} in vector constant {} of length {}",
                        idx,
                        name,
                        vector.len()
                    )))
                }
            }
            _ => Err(SemanticError::InvalidUsage(format!(
                "Identifier {} was declared as {} which is not a supported type.",
                name, vector_access_type
            ))),
        }
    }

    /// Checks that the specified name and index are a valid reference to a declared matrix constant.
    /// If not, it returns an error.
    ///
    /// # Errors
    /// - Returns an error if the identifier is not associated with a matrix access type.
    /// - Returns an error if the row index is greater than the matrix row length.
    /// - Returns an error if the column index is greater than the matrix column length.
    pub(super) fn validate_matrix_access(
        &self,
        name: &str,
        row_idx: usize,
        col_idx: usize,
    ) -> Result<(), SemanticError> {
        let matrix_access_type = self.get_type(name)?;
        match matrix_access_type {
            IdentifierType::Constant(ConstantType::Matrix(matrix)) => {
                if row_idx >= matrix.len() {
                    return Err(SemanticError::IndexOutOfRange(format!(
                        "Out-of-range index [{}] in matrix constant {} of row length {}",
                        row_idx,
                        name,
                        matrix.len()
                    )));
                }
                if col_idx >= matrix[0].len() {
                    return Err(SemanticError::IndexOutOfRange(format!(
                        "Out-of-range index [{}][{}] in matrix constant {} of column length {}",
                        row_idx,
                        col_idx,
                        name,
                        matrix[0].len()
                    )));
                }
                Ok(())
            }
            _ => Err(SemanticError::InvalidUsage(format!(
                "Identifier {} was declared as {} which is not a supported type.",
                name, matrix_access_type
            ))),
        }
    }
}

/// Validates the cycle length of the specified periodic column.
pub(super) fn validate_cycles(column: &PeriodicColumn) -> Result<(), SemanticError> {
    let name = column.name();
    let cycle = column.values().len();

    if !cycle.is_power_of_two() {
        return Err(SemanticError::InvalidPeriodicColumn(format!(
            "cycle length must be a power of two, but was {} for cycle {}",
            cycle, name
        )));
    }

    if cycle < MIN_CYCLE_LENGTH {
        return Err(SemanticError::InvalidPeriodicColumn(format!(
            "cycle length must be at least {}, but was {} for cycle {}",
            MIN_CYCLE_LENGTH, cycle, name
        )));
    }

    Ok(())
}

// HELPERS
// ================================================================================================

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
