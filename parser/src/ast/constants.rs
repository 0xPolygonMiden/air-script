// CONSTANTS
// ================================================================================================

use super::Identifier;

/// Stores a constant's name and value. There are three types of constants:
/// - Scalar: 123
/// - Vector: \[1, 2, 3\]
/// - Matrix: \[\[1, 2, 3\], \[4, 5, 6\]\]
#[derive(Debug, PartialEq, Eq)]
pub struct Constant {
    name: Identifier,
    value: ConstantType,
}

impl Constant {
    /// Returns a new instance of a [Constant]
    pub fn new(name: Identifier, value: ConstantType) -> Self {
        Self { name, value }
    }

    /// Returns the name of the [Constant]
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns the value of the [Constant]
    pub fn value(&self) -> &ConstantType {
        &self.value
    }
}

/// Type of constant. Constants can be of 3 types:
/// - Scalar: 123
/// - Vector: \[1, 2, 3\]
/// - Matrix: \[\[1, 2, 3\], \[4, 5, 6\]\]
#[derive(Debug, PartialEq, Eq)]
pub enum ConstantType {
    Scalar(u64),
    Vector(Vec<u64>),
    Matrix(Vec<Vec<u64>>),
}
