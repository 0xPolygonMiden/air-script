use super::Identifier;
use std::fmt::Display;

/// Defines the type of an access into a binding such as a [ConstantBinding] or a [VariableBinding].
///
/// - Default: accesses the entire bound value, which could be a scalar, vector, or matrix.
/// - Vector: indexes into the bound value at the specified index. The result could be either a
///   single value or a vector, depending on the type of the original binding. This is not allowed
///   for bindings to scalar values and will result in an error.
/// - Matrix: indexes into the bound value at the specified row and column. The result is a single
///   value. This [AccessType] is not allowed for bindings to scalar or vector values and will
///   result in an error.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum AccessType {
    Default,
    Slice(Range),
    Vector(usize),
    /// Access into a matrix, with the values referring to the row and column indices respectively.
    Matrix(usize, usize),
}

impl Display for AccessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "direct reference by name"),
            Self::Slice(_) => write!(f, "slice"),
            Self::Vector(_) => write!(f, "vector"),
            Self::Matrix(_, _) => write!(f, "matrix"),
        }
    }
}

/// [SymbolAccess] is used to indicate referencing all or part of an identifier that is bound to a
/// value, such as a [ConstantBinding] or a [VariableBinding].
///
/// - `name`: is the identifier of the [ConstantBinding] or [VariableBinding] being accessed.
/// - `access_type`: specifies the [AccessType] by which the identifier is being accessed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolAccess {
    name: Identifier,
    access_type: AccessType,
    offset: usize,
}

impl SymbolAccess {
    pub fn new(name: Identifier, access_type: AccessType, offset: usize) -> Self {
        Self {
            name,
            access_type,
            offset,
        }
    }

    pub fn ident(&self) -> &Identifier {
        &self.name
    }

    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// Gets the access type of this [SymbolAccess].
    pub fn access_type(&self) -> &AccessType {
        &self.access_type
    }

    /// Gets the offset of this [SymbolAccess].
    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn into_parts(self) -> (Identifier, AccessType, usize) {
        (self.name, self.access_type, self.offset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Range {
    start: usize,
    end: usize,
}

impl Range {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}

/// Contains values to be iterated over in a comprehension such as list comprehension or constraint
/// comprehension.
///
/// For example, in the list comprehension \[x + y + z for (x, y, z) in (x, 0..5, z\[1..6\])\],
/// `x` is an Iterable of type Identifier representing the vector to iterate over,
/// `0..5` is an Iterable of type Range representing the range to iterate over,
/// `z[1..6]` is an Iterable of type Slice representing the slice of the vector z to iterate over.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Iterable {
    Identifier(Identifier),
    Range(Range),
    Slice(Identifier, Range),
}
