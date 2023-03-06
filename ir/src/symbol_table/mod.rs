use super::{
    ast, BTreeMap, Constant, ConstantType, Declarations, Identifier, IndexedTraceAccess,
    MatrixAccess, NamedTraceAccess, SemanticError, TraceSegment, Variable, VariableType,
    VectorAccess, MIN_CYCLE_LENGTH,
};
use std::fmt::Display;

mod trace_columns;
use trace_columns::TraceColumns;

mod access_validation;
use access_validation::ValidateIdentifierAccess;

// TYPES
// ================================================================================================

/// TODO: docs
/// TODO: get rid of need to make this public to the crate
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum IdentifierType {
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

// TODO: get rid of need to make this public
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum Scope {
    BoundaryConstraints,
    IntegrityConstraints,
}

// TODO: get rid of need to make this public
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum VariableValue {
    Scalar(String),
    Vector(VectorAccess),
    Matrix(MatrixAccess),
}

// SYMBOL TABLE
// ================================================================================================

/// SymbolTable for identifiers to track their types and information and enforce uniqueness of
/// identifiers.
#[derive(Default, Debug)]
pub struct SymbolTable {
    /// A map of all declared identifiers from their name (the key) to their type.
    identifiers: BTreeMap<String, IdentifierType>,

    /// TODO: docs
    declarations: Declarations,
}

impl SymbolTable {
    /// Consumes this symbol table and returns the information required for declaring constants,
    /// public inputs, periodic columns and columns amount for the AIR.
    pub(super) fn into_declarations(self) -> Declarations {
        self.declarations
    }

    // --- MUTATORS -------------------------------------------------------------------------------

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
            Some(prev_type) => Err(SemanticError::duplicate_identifer(
                ident_name, ident_type, prev_type,
            )),
            None => Ok(()),
        }
    }

    /// Add a constant by its identifier and value.
    pub(super) fn insert_constant(&mut self, constant: &Constant) -> Result<(), SemanticError> {
        validate_constant(constant)?;

        let Identifier(name) = &constant.name();
        self.insert_symbol(name, IdentifierType::Constant(constant.value().clone()))?;
        self.declarations.add_constant(constant.clone());

        Ok(())
    }

    /// Add all trace columns in the specified trace segment by their identifiers, sizes and indices.
    pub(super) fn insert_trace_columns(
        &mut self,
        trace_segment: TraceSegment,
        trace: &[ast::TraceCols],
    ) -> Result<(), SemanticError> {
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

        if col_idx > u16::MAX.into() {
            return Err(SemanticError::InvalidTraceSegment(format!(
                "Trace segment {} has {} columns, but the maximum number of columns is {}",
                trace_segment,
                col_idx,
                u16::MAX
            )));
        }

        self.declarations
            .set_trace_segment_width(usize::from(trace_segment), col_idx as u16);

        Ok(())
    }

    /// Adds all public inputs by their identifier names and array length.
    pub(super) fn insert_public_inputs(
        &mut self,
        public_inputs: &[ast::PublicInput],
    ) -> Result<(), SemanticError> {
        for input in public_inputs.iter() {
            self.insert_symbol(input.name(), IdentifierType::PublicInput(input.size()))?;
            self.declarations
                .add_public_input((input.name().to_string(), input.size()));
        }

        Ok(())
    }

    /// Adds all periodic columns by their identifier names, their indices in the array of all
    /// periodic columns, and the lengths of their periodic cycles.
    pub(super) fn insert_periodic_columns(
        &mut self,
        columns: &[ast::PeriodicColumn],
    ) -> Result<(), SemanticError> {
        for (index, column) in columns.iter().enumerate() {
            validate_cycles(column)?;
            let values = column.values().to_vec();

            self.insert_symbol(
                column.name(),
                IdentifierType::PeriodicColumn(index, values.len()),
            )?;
            self.declarations.add_periodic_column(values);
        }

        Ok(())
    }

    /// Adds all random values by their identifier names and array length.
    pub(super) fn insert_random_values(
        &mut self,
        values: &ast::RandomValues,
    ) -> Result<(), SemanticError> {
        self.declarations
            .set_num_random_values(values.size() as u16);
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

    // --- ACCESSORS ------------------------------------------------------------------------------

    /// Gets the number of trace segments that were specified for this AIR.
    pub(super) fn num_trace_segments(&self) -> usize {
        self.declarations.num_trace_segments()
    }

    /// Returns the type associated with the specified identifier name.
    ///
    /// # Errors
    /// Returns an error if the identifier was not in the symbol table.
    pub(crate) fn get_type(&self, name: &str) -> Result<&IdentifierType, SemanticError> {
        if let Some(ident_type) = self.identifiers.get(name) {
            Ok(ident_type)
        } else {
            Err(SemanticError::undeclared_identifier(name))
        }
    }

    /// Looks up a [NamedTraceAccess] by its identifier name and returns an equivalent
    /// [IndexedTraceAccess].
    ///
    /// # Errors
    /// Returns an error if:
    /// - the identifier was not in the symbol table.
    /// - the identifier was not declared as a trace column binding.
    /// TODO: update docs
    pub(crate) fn get_trace_access_by_name(
        &self,
        trace_access: &NamedTraceAccess,
    ) -> Result<IndexedTraceAccess, SemanticError> {
        let symbol_type = self.get_type(trace_access.name())?;
        trace_access.validate(symbol_type)?;

        let IdentifierType::TraceColumns(columns) = symbol_type else { unreachable!("validation of named trace access failed.") };
        Ok(IndexedTraceAccess::new(
            columns.trace_segment(),
            columns.offset() + trace_access.idx(),
            trace_access.row_offset(),
        ))
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
    /// TODO: update docs
    pub(crate) fn access_vector_element(
        &self,
        vector_access: &VectorAccess,
    ) -> Result<&IdentifierType, SemanticError> {
        let symbol_type = self.get_type(vector_access.name())?;
        vector_access.validate(symbol_type)?;

        Ok(symbol_type)
    }

    /// Checks that the specified name and index are a valid reference to a matrix constant and
    /// returns the symbol type. If it's not a valid reference, an error is returned.
    ///
    /// # Errors
    /// - Returns an error if the identifier is not in the symbol table.
    /// - Returns an error if the identifier is not associated with a matrix access type.
    /// - Returns an error if the row index is greater than the matrix row length.
    /// - Returns an error if the column index is greater than the matrix column length.
    /// TODO: update docs
    pub(crate) fn access_matrix_element(
        &self,
        matrix_access: &MatrixAccess,
    ) -> Result<&IdentifierType, SemanticError> {
        let symbol_type = self.get_type(matrix_access.name())?;
        matrix_access.validate(symbol_type)?;

        Ok(symbol_type)
    }

    // --- VALIDATION -----------------------------------------------------------------------------

    /// Checks that the specified trace access is valid, i.e. that it references a declared trace
    /// segment and the index is within the bounds of the declared segment width.
    ///
    /// # Errors
    /// Returns an error if:
    /// - the specified trace segment is out of range.
    /// - the specified column index is out of range.
    pub(crate) fn validate_trace_access(
        &self,
        trace_access: &IndexedTraceAccess,
    ) -> Result<(), SemanticError> {
        let trace_segment = usize::from(trace_access.trace_segment());
        let trace_segment_width = self.declarations.trace_segment_width(trace_segment)?;

        if trace_access.col_idx() as u16 >= trace_segment_width {
            return Err(SemanticError::indexed_trace_column_access_out_of_bounds(
                trace_access,
                trace_segment_width,
            ));
        }

        Ok(())
    }

    /// Checks that the specified random value access index is valid, i.e. that it is in range of
    /// the number of declared random values.
    pub(crate) fn validate_rand_access(&self, index: usize) -> Result<(), SemanticError> {
        if index >= usize::from(self.declarations.num_random_values()) {
            return Err(SemanticError::random_value_access_out_of_bounds(
                index,
                self.declarations.num_random_values(),
            ));
        }

        Ok(())
    }
}

// HELPERS
// ================================================================================================

/// Validates the cycle length of the specified periodic column.
fn validate_cycles(column: &ast::PeriodicColumn) -> Result<(), SemanticError> {
    let name = column.name();
    let cycle = column.values().len();

    if !cycle.is_power_of_two() {
        return Err(SemanticError::periodic_cycle_length_not_power_of_two(
            cycle, name,
        ));
    }

    if cycle < MIN_CYCLE_LENGTH {
        return Err(SemanticError::periodic_cycle_length_too_small(cycle, name));
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
                Err(SemanticError::invalid_matrix_constant(constant))
            }
        }
        _ => Ok(()),
    }
}