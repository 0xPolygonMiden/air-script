use super::{
    ast, BTreeMap, Constant, ConstantType, Declarations, Identifier, IndexedTraceAccess,
    MatrixAccess, NamedTraceAccess, SemanticError, TraceSegment, Variable, VariableType,
    VectorAccess, CURRENT_ROW, MIN_CYCLE_LENGTH,
};

mod symbol;
pub(crate) use symbol::{Scope, Symbol, SymbolType};

mod symbol_access;
pub(crate) use symbol_access::AccessType;
use symbol_access::ValidateIdentifierAccess;

mod trace_columns;
use trace_columns::TraceColumns;

mod value;
pub use value::{ConstantValue, Value};

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
        name: String,
        scope: Scope,
        symbol_type: SymbolType,
    ) -> Result<(), SemanticError> {
        // check if the identifier was already defined in the global scope.
        if let Some(global_sym) = self.symbols.get(&(name.clone(), Scope::Global)) {
            // TODO: update error to "{name} was already defined as a {type} in the {scope}",
            return Err(SemanticError::duplicate_identifer(
                &name,
                &symbol_type,
                global_sym.symbol_type(),
            ));
        }

        // insert the identifier or return an error if it was already defined in the specified scope.
        let symbol = Symbol::new(name.clone(), scope, symbol_type);
        if let Some(prev_symbol) = self.symbols.insert((name, scope), symbol.clone()) {
            // TODO: update error to "{name} was already defined as a {type} in the {scope}",
            return Err(SemanticError::duplicate_identifer(
                symbol.name(),
                symbol.symbol_type(),
                prev_symbol.symbol_type(),
            ));
        }

        Ok(())
    }

    /// Add a constant by its identifier and value.
    pub(super) fn insert_constant(&mut self, constant: Constant) -> Result<(), SemanticError> {
        validate_constant(&constant)?;
        self.declarations.add_constant(constant.clone());

        let (name, constant_type) = constant.into_parts();
        self.insert_symbol(name, Scope::Global, SymbolType::Constant(constant_type))?;

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
                trace_cols.name().to_string(),
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
        public_inputs: Vec<ast::PublicInput>,
    ) -> Result<(), SemanticError> {
        for input in public_inputs.into_iter() {
            let (name, size) = input.into_parts();
            self.insert_symbol(
                name.clone(),
                Scope::BoundaryConstraints,
                SymbolType::PublicInput(size),
            )?;
            self.declarations.add_public_input((name, size));
        }

        Ok(())
    }

    /// Adds all periodic columns by their identifier names, their indices in the array of all
    /// periodic columns, and the lengths of their periodic cycles.
    pub(super) fn insert_periodic_columns(
        &mut self,
        columns: Vec<ast::PeriodicColumn>,
    ) -> Result<(), SemanticError> {
        for (index, column) in columns.into_iter().enumerate() {
            validate_cycles(&column)?;

            let (name, values) = column.into_parts();
            self.insert_symbol(
                name,
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
        rand_values: ast::RandomValues,
    ) -> Result<(), SemanticError> {
        let (name, num_values, bindings) = rand_values.into_parts();

        let mut offset = 0;
        // add the name of the random values array to the symbol table
        self.insert_symbol(
            name,
            Scope::Global,
            SymbolType::RandomValuesBinding(offset, num_values as usize),
        )?;

        // add the named random value bindings to the symbol table
        for binding in bindings {
            let (name, size) = binding.into_parts();
            self.insert_symbol(
                name,
                Scope::Global,
                SymbolType::RandomValuesBinding(offset, size as usize),
            )?;
            offset += size as usize;
        }

        // TODO: check this type coercion
        self.declarations.set_num_random_values(num_values as u16);

        Ok(())
    }

    /// Inserts a boundary variable into the symbol table.
    pub(super) fn insert_boundary_variable(
        &mut self,
        variable: Variable,
    ) -> Result<(), SemanticError> {
        let (name, value) = variable.into_parts();
        self.insert_symbol(
            name,
            Scope::BoundaryConstraints,
            SymbolType::Variable(value),
        )?;
        Ok(())
    }

    /// Inserts an integrity variable into the symbol table.
    pub(super) fn insert_integrity_variable(
        &mut self,
        name: String,
        variable_type: VariableType,
    ) -> Result<(), SemanticError> {
        self.insert_symbol(
            name,
            Scope::IntegrityConstraints,
            SymbolType::Variable(variable_type),
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
        &mut self,
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
