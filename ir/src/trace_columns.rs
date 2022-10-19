use super::SemanticError;
use std::collections::BTreeMap;

/// A representation of the trace columns declared for the AIR.
#[derive(Debug, Default)]
pub struct TraceColumns {
    /// A map of a set of trace columns using the declared identifier as the key and the column
    /// index as the value.
    columns: BTreeMap<String, usize>,
}

impl TraceColumns {
    /// Add a new column by its name and index
    pub(super) fn insert(&mut self, name: &str, index: usize) -> Result<(), SemanticError> {
        let result = self.columns.insert(name.to_string(), index);
        if result.is_some() {
            Err(SemanticError::DuplicateTraceColumn(format!(
                "{} was defined more than once",
                name
            )))
        } else {
            Ok(())
        }
    }

    /// Returns the index in the trace of the column with the specified name.
    pub(crate) fn get_column_index(&self, name: &str) -> Result<usize, SemanticError> {
        if let Some(&index) = self.columns.get(name) {
            Ok(index)
        } else {
            Err(SemanticError::UndefinedIdentifier(format!(
                "{} has not been declared",
                name,
            )))
        }
    }
}
