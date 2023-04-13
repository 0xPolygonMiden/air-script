use super::Identifier;

// CONSTANTS
// ================================================================================================

/// Stores a constant's name and value. There are three types of constants:
/// - Scalar: 123
/// - Vector: \[1, 2, 3\]
/// - Matrix: \[\[1, 2, 3\], \[4, 5, 6\]\]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantBinding {
    name: Identifier,
    value: ConstantType,
}

impl ConstantBinding {
    /// Returns a new instance of a [ConstantBinding]
    pub fn new(name: Identifier, value: ConstantType) -> Self {
        Self { name, value }
    }

    /// Returns the name of the [ConstantBinding]
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns the value of the [ConstantBinding]
    pub fn value(&self) -> &ConstantType {
        &self.value
    }

    pub fn into_parts(self) -> (String, ConstantType) {
        (self.name.into_name(), self.value)
    }
}

/// Type of constant. Constants can be of 3 types:
/// - Scalar: 123
/// - Vector: \[1, 2, 3\]
/// - Matrix: \[\[1, 2, 3\], \[4, 5, 6\]\]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConstantType {
    Scalar(u64),
    Vector(Vec<u64>),
    Matrix(Vec<Vec<u64>>),
}
