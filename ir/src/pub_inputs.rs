use super::SemanticError;
use std::collections::BTreeMap;

/// A representation of the public inputs declared for an Air.
#[derive(Debug, Default)]
pub struct PublicInputs {
    /// A map of the Air's public inputs the declared identifier as the key and the size of the
    /// input array as the value.
    inputs: BTreeMap<String, usize>,
}

impl PublicInputs {
    /// Adds a new public input by its name and size.
    pub(super) fn insert(&mut self, name: &str, size: usize) -> Result<(), SemanticError> {
        let result = self.inputs.insert(name.to_string(), size);
        if result.is_some() {
            Err(SemanticError::DuplicatePublicInput(format!(
                "{} was defined more than once",
                name
            )))
        } else {
            Ok(())
        }
    }

    /// Checks that the specified name and index are a valid reference to a declared public input or
    /// returns an error.
    ///
    /// # Errors
    /// - Returns an error if the name was not declared as a public input.
    /// - Returns an error if the index is not in the declared public input array.
    pub(super) fn validate_input(&self, name: &str, index: usize) -> Result<(), SemanticError> {
        let pub_input = self.inputs.get(name);
        match pub_input {
            None => Err(SemanticError::InvalidIdentifier(format!(
                "Public input {} was not declared",
                name
            ))),
            Some(array_len) => {
                if index < *array_len {
                    Ok(())
                } else {
                    Err(SemanticError::IndexOutOfRange(format!(
                        "Out-of-range index {} in public input {} of length {}",
                        index, name, array_len
                    )))
                }
            }
        }
    }
}
