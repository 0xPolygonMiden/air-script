use super::{
    BTreeMap, PeriodicColumns, PublicInputs, SemanticError, TraceSegment, MIN_CYCLE_LENGTH,
};
use parser::ast::{Identifier, PeriodicColumn, PublicInput};
use std::fmt::Display;

#[derive(Debug, Copy, Clone)]
pub(super) enum IdentifierType {
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
        let result = self.identifiers.insert(ident_name.to_owned(), ident_type);
        match result {
            Some(prev_type) => Err(SemanticError::DuplicateIdentifier(format!(
                "Cannot declare {} as a {}, since it was already defined as a {}",
                ident_name, ident_type, prev_type
            ))),
            None => Ok(()),
        }
    }

    // --- PUBLIC MUTATORS ------------------------------------------------------------------------

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

    /// Consumes this symbol table and returns the information required for declaring public inputs
    /// and periodic columns for the AIR.
    pub(super) fn into_declarations(self) -> (PublicInputs, PeriodicColumns) {
        (self.public_inputs, self.periodic_columns)
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
            Ok(*ident_type)
        } else {
            Err(SemanticError::InvalidIdentifier(format!(
                "Identifier {} was not declared",
                name
            )))
        }
    }

    /// Checks that the specified name and index are a valid reference to a declared public input or
    /// returns an error.
    ///
    /// # Errors
    /// - Returns an error if the name was not declared as a public input.
    /// - Returns an error if the index is not in the declared public input array.
    pub(super) fn validate_public_input(
        &self,
        name: &str,
        index: usize,
    ) -> Result<(), SemanticError> {
        let ident_type = self.get_type(name)?;
        if let IdentifierType::PublicInput(size) = ident_type {
            if index < size {
                Ok(())
            } else {
                Err(SemanticError::IndexOutOfRange(format!(
                    "Out-of-range index {} in public input {} of length {}",
                    index, name, size
                )))
            }
        } else {
            Err(SemanticError::InvalidUsage(format!(
                "Identifier {} was declared as {}, not as a public input",
                name, ident_type
            )))
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
