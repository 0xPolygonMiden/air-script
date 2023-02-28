use super::{
    ast, BTreeMap, Constant, ConstantType, Declarations, Identifier, IndexedTraceAccess,
    MatrixAccess, NamedTraceAccess, SemanticError, TraceSegment, Variable, VariableType,
    VectorAccess, MIN_CYCLE_LENGTH,
};

mod symbol;
use symbol::Symbol;
pub(crate) use symbol::{Scope, SymbolType};

mod symbol_access;
pub(crate) use symbol_access::SymbolAccess;
use symbol_access::ValidateIdentifierAccess;

mod trace_columns;
use trace_columns::TraceColumns;

// TYPES
// ================================================================================================

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
    symbols: BTreeMap<(String, Scope), Symbol>,

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

    /// Adds a declared identifier to the symbol table using the identifier and scope as the key and
    /// the symbol as the value, where the symbol contains relevant details like scope and type.
    ///
    /// # Errors
    /// It returns an error if the identifier already existed in the table in the same scope or the
    /// global scope.
    fn insert_symbol(
        &mut self,
        name: &str,
        scope: Scope,
        symbol_type: SymbolType,
    ) -> Result<(), SemanticError> {
        let symbol = Symbol::new(name.to_string(), scope, symbol_type);

        // check if the identifier was already defined in the global scope.
        if let Some(global_sym) = self.symbols.get(&(name.to_owned(), Scope::Global)) {
            // TODO: update error to "{name} was already defined as a {type} in the {scope}",
            return Err(SemanticError::duplicate_identifer(
                symbol.name(),
                symbol.symbol_type(),
                global_sym.symbol_type(),
            ));
        }
        // insert the identifier or return an error if it was already defined in the specified scope.
        if let Some(symbol) = self.symbols.insert((name.to_owned(), scope), symbol) {
            // TODO: update error to "{name} was already defined as a {type} in the {scope}",
            return Err(SemanticError::duplicate_identifer(
                symbol.name(),
                symbol.symbol_type(),
                symbol.symbol_type(),
            ));
        }

        Ok(())
    }

    /// Add a constant by its identifier and value.
    /// TODO: consume constants instead of cloning them
    pub(super) fn insert_constant(&mut self, constant: &Constant) -> Result<(), SemanticError> {
        let Identifier(name) = &constant.name();
        validate_constant(constant)?;
        self.insert_symbol(
            name,
            Scope::Global,
            SymbolType::Constant(constant.value().clone()),
        )?;
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
                Scope::Global,
                SymbolType::TraceColumns(trace_columns),
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
            self.insert_symbol(
                input.name(),
                Scope::BoundaryConstraints,
                SymbolType::PublicInput(input.size()),
            )?;
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
                Scope::IntegrityConstraints,
                SymbolType::PeriodicColumn(index, values.len()),
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
            Scope::Global,
            SymbolType::RandomValuesBinding(offset, values.size() as usize),
        )?;
        // add the named random value bindings to the symbol table
        for value in values.bindings() {
            self.insert_symbol(
                value.name(),
                Scope::Global,
                SymbolType::RandomValuesBinding(offset, value.size() as usize),
            )?;
            offset += value.size() as usize;
        }
        Ok(())
    }

    /// Inserts a boundary variable into the symbol table.
    pub(super) fn insert_boundary_variable(
        &mut self,
        variable: &Variable,
    ) -> Result<(), SemanticError> {
        self.insert_symbol(
            variable.name(),
            Scope::BoundaryConstraints,
            SymbolType::Variable(variable.value().clone()),
        )?;
        Ok(())
    }

    /// Inserts an integrity variable into the symbol table.
    pub(super) fn insert_integrity_variable(
        &mut self,
        variable: &Variable,
    ) -> Result<(), SemanticError> {
        self.insert_symbol(
            variable.name(),
            Scope::IntegrityConstraints,
            SymbolType::Variable(variable.value().clone()),
        )?;
        Ok(())
    }

    // --- ACCESSORS ------------------------------------------------------------------------------

    /// Gets the number of trace segments that were specified for this AIR.
    pub(super) fn num_trace_segments(&self) -> usize {
        self.declarations.num_trace_segments()
    }

    /// Returns the symbol associated with the specified identifier and validated for the specified
    /// constraint domain.
    ///
    /// # Errors
    /// Returns an error if the identifier was not in the symbol table.
    pub(super) fn get_symbol(&self, name: &str, scope: Scope) -> Result<&Symbol, SemanticError> {
        if let Some(symbol) = self
            .symbols
            .get(&(name.to_owned(), scope))
            .or_else(|| self.symbols.get(&(name.to_owned(), Scope::Global)))
        {
            Ok(symbol)
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
        let symbol = self.get_symbol(trace_access.name(), Scope::Global)?;
        trace_access.validate(symbol)?;

        let SymbolType::TraceColumns(columns) = symbol.symbol_type() else { unreachable!("validation of named trace access failed.") };
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
        scope: Scope,
    ) -> Result<SymbolAccess, SemanticError> {
        let symbol = self.get_symbol(vector_access.name(), scope)?;
        SymbolAccess::from_vector_access(symbol, vector_access)
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
        scope: Scope,
    ) -> Result<SymbolAccess, SemanticError> {
        let symbol = self.get_symbol(matrix_access.name(), scope)?;
        SymbolAccess::from_matrix_access(symbol, matrix_access)
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
