// CONSTANTS
// ================================================================================================

use super::Identifier;

/// Stores a constant's name and value. There are three types of constants:
/// - Scalar: 1, 2, 3
/// - Vector: \[1, 2, 3\]
/// - Matrix: \[\[1, 2, 3\], \[4, 5, 6\]\]
#[derive(Debug, PartialEq, Eq)]
pub struct Constant {
    pub name: Identifier,
    pub value: ConstantType,
}

impl Constant {
    /// Returns a new instance of a [Constant]
    pub fn new(name: Identifier, value: ConstantType) -> Self {
        Self { name, value }
    }
}

/// Type of constant. Constants can be of 3 types:
/// - Scalar: 1, 2, 3
/// - Vector: \[1, 2, 3\]
/// - Matrix: \[\[1, 2, 3\], \[4, 5, 6\]\]
#[derive(Debug, PartialEq, Eq)]
pub enum ConstantType {
    Scalar(u64),
    Vector(Vec<u64>),
    Matrix(Vec<Vec<u64>>),
}
