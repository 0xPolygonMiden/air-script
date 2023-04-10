use super::{
    ast, BTreeMap, Constant, ConstantType, Declarations, Identifier, MatrixAccess, SemanticError,
    TraceAccess, TraceBinding, TraceBindingAccess, Variable, VariableType, VectorAccess,
    CURRENT_ROW, MIN_CYCLE_LENGTH,
};

mod symbol;
pub(crate) use symbol::Symbol;

mod symbol_access;
use symbol_access::ValidateIdentifierAccess;
pub(crate) use symbol_access::{AccessType, ValidateAccess};

mod symbol_type;
pub(crate) use symbol_type::SymbolType;

mod value;
pub use value::{ConstantValue, Value};

// SYMBOL TABLE
// ================================================================================================

/// SymbolTable for identifiers to track their types and information and enforce uniqueness of
/// identifiers.
#[derive(Default, Debug)]
pub struct SymbolTable {
    /// A map of all declared identifiers from their name (the key) to their type.
    symbols: BTreeMap<String, Symbol>,

    /// TODO: docs
    variables: Vec<String>,

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

    /// TODO: docs
    pub(crate) fn clear_variables(&mut self) {
        for variable in self.variables.iter() {
            self.symbols.remove(variable);
        }
        self.variables.clear();
    }

    /// Add a constant by its identifier and value.
    pub(super) fn insert_constant(&mut self, constant: Constant) -> Result<(), SemanticError> {
        self.declarations.add_constant(constant.clone());
        let (name, constant_type) = constant.into_parts();

        // check the number of elements in each row are same for a matrix
        if let ConstantType::Matrix(matrix) = &constant_type {
            let row_len = matrix[0].len();
            if matrix.iter().skip(1).any(|row| row.len() != row_len) {
                return Err(SemanticError::invalid_matrix_constant(&name));
            }
        }

        self.insert_symbol(name, SymbolType::Constant(constant_type))?;

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
            self.insert_symbol(name, SymbolType::PeriodicColumn(index, values.len()))?;
            self.declarations.add_periodic_column(values);
        }

        Ok(())
    }

    /// Adds all public inputs by their identifier names and array length.
    pub(super) fn insert_public_inputs(
        &mut self,
        public_inputs: Vec<ast::PublicInput>,
    ) -> Result<(), SemanticError> {
        for input in public_inputs.into_iter() {
            let (name, size) = input.into_parts();
            self.insert_symbol(name.clone(), SymbolType::PublicInput(size))?;
            self.declarations.add_public_input((name, size));
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
            SymbolType::RandomValuesBinding(offset, num_values as usize),
        )?;

        // add the named random value bindings to the symbol table
        for binding in bindings {
            let (name, size) = binding.into_parts();
            self.insert_symbol(name, SymbolType::RandomValuesBinding(offset, size as usize))?;
            offset += size as usize;
        }

        // TODO: check this type coercion
        self.declarations.set_num_random_values(num_values as u16);

        Ok(())
    }

    /// Add all trace columns in the specified trace segment by their identifiers, sizes and indices.
    pub(super) fn insert_trace_bindings(
        &mut self,
        trace: Vec<Vec<TraceBinding>>,
    ) -> Result<(), SemanticError> {
        for (trace_segment, bindings) in trace.into_iter().enumerate() {
            let mut width = 0;
            for binding in bindings {
                width = binding.offset() + binding.size();
                self.insert_symbol(
                    binding.name().to_string(),
                    SymbolType::TraceColumns(binding),
                )?;
            }

            if width > u16::MAX.into() {
                return Err(SemanticError::InvalidTraceSegment(format!(
                    "Trace segment {} has {} columns, but the maximum number of columns is {}",
                    trace_segment,
                    width,
                    u16::MAX
                )));
            }

            self.declarations
                .set_trace_segment_width(trace_segment, width as u16);
        }

        Ok(())
    }

    /// Inserts a variable into the symbol table.
    pub(super) fn insert_variable(&mut self, variable: Variable) -> Result<(), SemanticError> {
        let (name, value) = variable.into_parts();
        self.insert_symbol(name, SymbolType::Variable(value))?;
        Ok(())
    }

    /// Adds a declared identifier to the symbol table using the identifier and scope as the key and
    /// the symbol as the value, where the symbol contains relevant details like scope and type.
    ///
    /// # Errors
    /// It returns an error if the identifier already existed in the table in the same scope or the
    /// global scope.
    fn insert_symbol(
        &mut self,
        name: String,
        symbol_type: SymbolType,
    ) -> Result<(), SemanticError> {
        // insert the identifier or return an error if it was already defined.
        let symbol = Symbol::new(name.clone(), symbol_type.clone());

        if let Some(symbol) = self.symbols.insert(name.clone(), symbol) {
            return Err(SemanticError::duplicate_identifer(
                &name,
                &symbol_type,
                symbol.symbol_type(),
            ));
        } else if matches!(symbol_type, SymbolType::Variable(_)) {
            // track variables so we can clear them out when we are done with them
            self.variables.push(name);
        }

        Ok(())
    }

    // --- ACCESSORS ------------------------------------------------------------------------------

    /// Returns the symbol associated with the specified identifier and validated for the specified
    /// constraint domain.
    ///
    /// # Errors
    /// Returns an error if the identifier was not in the symbol table.
    pub(super) fn get_symbol(&self, name: &str) -> Result<&Symbol, SemanticError> {
        if let Some(symbol) = self.symbols.get(name) {
            Ok(symbol)
        } else {
            Err(SemanticError::undeclared_identifier(name))
        }
    }

    /// Looks up a [TraceBindingAccess] by its identifier name and returns an equivalent
    /// [TraceAccess].
    ///
    /// # Errors
    /// Returns an error if:
    /// - the identifier was not in the symbol table.
    /// - the identifier was not declared as a trace column binding.
    /// TODO: update docs
    pub(crate) fn get_trace_binding_access(
        &self,
        trace_access: &TraceBindingAccess,
    ) -> Result<TraceAccess, SemanticError> {
        let symbol = self.get_symbol(trace_access.name())?;
        trace_access.validate(symbol)?;

        let SymbolType::TraceColumns(columns) = symbol.symbol_type() else { unreachable!("validation of named trace access failed.") };
        Ok(TraceAccess::new(
            columns.trace_segment(),
            columns.offset() + trace_access.col_offset(),
            1,
            trace_access.row_offset(),
        ))
    }

    /// Gets the number of trace segments that were specified for this AIR.
    pub(super) fn num_trace_segments(&self) -> usize {
        self.declarations.num_trace_segments()
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
        trace_access: &TraceAccess,
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
