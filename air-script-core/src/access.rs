use super::Identifier;

/// [VectorAccess] is used to represent an element inside vector at the specified index.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct VectorAccess {
    name: Identifier,
    idx: usize,
}

impl VectorAccess {
    /// Creates a new [VectorAccess] instance with the specified identifier name and index.
    pub fn new(name: Identifier, idx: usize) -> Self {
        Self { name, idx }
    }

    /// Returns the name of the vector.
    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// Returns the index of the vector access.
    pub fn idx(&self) -> usize {
        self.idx
    }
}

/// [MatrixAccess] is used to represent an element inside a matrix at the specified row and column
/// indices.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct MatrixAccess {
    name: Identifier,
    row_idx: usize,
    col_idx: usize,
}

impl MatrixAccess {
    /// Creates a new [MatrixAccess] instance with the specified identifier name and indices.
    pub fn new(name: Identifier, row_idx: usize, col_idx: usize) -> Self {
        Self {
            name,
            row_idx,
            col_idx,
        }
    }

    /// Returns the name of the matrix.
    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// Returns the row index of the matrix access.
    pub fn row_idx(&self) -> usize {
        self.row_idx
    }

    /// Returns the column index of the matrix access.
    pub fn col_idx(&self) -> usize {
        self.col_idx
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
