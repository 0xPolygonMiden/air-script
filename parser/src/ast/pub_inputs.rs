use super::Identifier;

// PUBLIC INPUTS
// ================================================================================================

/// Declaration of a public input for an AIR. Public inputs are represented by a named identifier
/// which is used to identify a fixed size array of length `size`.
#[derive(Debug, Eq, PartialEq)]
pub struct PublicInput {
    name: Identifier,
    size: usize,
}

impl PublicInput {
    pub(crate) fn new(name: Identifier, size: u64) -> Self {
        Self {
            name,
            size: size as usize,
        }
    }

    pub fn name(&self) -> &str {
        let Identifier(name) = &self.name;
        name
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
