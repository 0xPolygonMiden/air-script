use super::{BTreeMap, SemanticError, MIN_CYCLE_LENGTH};
use parser::ast::{Identifier, PeriodicColumn, PublicInput};
use std::fmt::Display;

#[derive(Debug, Copy, Clone)]
pub(super) enum IdentifierType {
    /// an identifier for a main trace column, containing its index in the main trace
    MainTraceColumn(usize),
    /// an identifier for a auxiliary trace column, containing its index in the auxiliary trace
    AuxTraceColumn(usize),
    /// an identifier for a public input, containing the size of the public input array
    PublicInput(usize),
    /// an identifier for a periodic column, containing its index out of all periodic columns
    PeriodicColumn(usize),
}

impl Display for IdentifierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PublicInput(_) => write!(f, "PublicInput"),
            Self::PeriodicColumn(_) => write!(f, "PeriodicColumn"),
            Self::MainTraceColumn(_) => write!(f, "MainTraceColumn"),
            Self::AuxTraceColumn(_) => write!(f, "AuxTraceColumn"),
        }
    }
}

/// SymbolTable for identifiers to track their types and information and enforce uniqueness of
/// identifiers.
#[derive(Default, Debug)]
pub(super) struct SymbolTable {
    /// A map of all declared identifiers from their name (the key) to their type.
    identifiers: BTreeMap<String, IdentifierType>,

    /// A map of the Air's periodic columns using the index of the column within the declared
    /// periodic columns as the key and the vector of periodic values as the value
    periodic_columns: Vec<Vec<u64>>,
}

impl SymbolTable {
    /// TODO
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

    // --- MUTATORS -------------------------------------------------------------------------------

    /// TODO Add a new main trace column by its name and index in the main execution trace.
    pub(super) fn insert_main_trace_columns(
        &mut self,
        columns: &[Identifier],
    ) -> Result<(), SemanticError> {
        for (idx, Identifier(name)) in columns.iter().enumerate() {
            self.insert_symbol(name, IdentifierType::MainTraceColumn(idx))?;
        }

        Ok(())
    }

    /// TODO Add a new auxiliary trace column by its name and index in the auxiliary execution trace.
    pub(super) fn insert_aux_trace_columns(
        &mut self,
        columns: &[Identifier],
    ) -> Result<(), SemanticError> {
        for (idx, Identifier(name)) in columns.iter().enumerate() {
            self.insert_symbol(name, IdentifierType::AuxTraceColumn(idx))?;
        }

        Ok(())
    }

    /// Adds a new public input by its name and size.
    pub(super) fn insert_public_inputs(
        &mut self,
        public_inputs: &[PublicInput],
    ) -> Result<(), SemanticError> {
        for input in public_inputs.iter() {
            self.insert_symbol(input.name(), IdentifierType::PublicInput(input.size()))?;
        }

        Ok(())
    }

    /// TODO
    pub(super) fn insert_periodic_columns(
        &mut self,
        columns: &[PeriodicColumn],
    ) -> Result<(), SemanticError> {
        for (index, column) in columns.iter().enumerate() {
            validate_cycles(column)?;

            self.insert_symbol(column.name(), IdentifierType::PeriodicColumn(index))?;
            self.periodic_columns.push(column.values().to_vec());
        }

        Ok(())
    }

    pub(super) fn into_periodic_columns(self) -> Vec<Vec<u64>> {
        self.periodic_columns
    }

    // --- ACCESSORS ------------------------------------------------------------------------------

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
