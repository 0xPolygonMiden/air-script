use std::fmt;

pub mod constants;
pub use constants::Constant;

pub mod pub_inputs;
pub use pub_inputs::PublicInput;

pub mod periodic_columns;
pub use periodic_columns::PeriodicColumn;

pub mod boundary_constraints;
pub use boundary_constraints::*;

pub mod transition_constraints;
pub use transition_constraints::*;

// AST
// ================================================================================================

/// [Source] is the root node of the AST representing the AIR constraints file.
#[derive(Debug, Eq, PartialEq)]
pub struct Source(pub Vec<SourceSection>);

/// Source is divided into SourceSections.
/// There are 6 types of Source Sections:
/// - AirDef: Name of the air constraints module.
/// - TraceCols: Trace Columns representing columns of the execution trace.
/// - PublicInputs: Public inputs are each represented by a fixed-size array. At least one public
///   input is required, but there is no limit to the number of public inputs that can be specified.
/// - PeriodicColumns: Periodic columns are each represented by a fixed-size array with all of its
///   elements specified. The array length is expected to be a power of 2, but this is not checked
///   during parsing.
/// - BoundaryConstraints: Boundary Constraints to be enforced on the boundaries of columns defined
///   in the TraceCols section. Currently there are two types of boundaries, First and Last
///   representing the first and last rows of the column.
/// - TransitionConstraints: Transition Constraints to be enforced on the trace columns defined
///   in the TraceCols section.
#[derive(Debug, Eq, PartialEq)]
pub enum SourceSection {
    AirDef(Identifier),
    Constant(Constant),
    Trace(Trace),
    PublicInputs(Vec<PublicInput>),
    PeriodicColumns(Vec<PeriodicColumn>),
    BoundaryConstraints(Vec<BoundaryStmt>),
    TransitionConstraints(Vec<TransitionStmt>),
}

// TRACE
// ================================================================================================

/// [Trace] contains the main and auxiliary trace segments of the execution trace.
#[derive(Debug, Eq, PartialEq)]
pub struct Trace {
    pub main_cols: Vec<TraceCols>,
    pub aux_cols: Vec<TraceCols>,
}

/// [TraceCols] is used to represent a single or a group of columns in the execution trace. For
/// single columns, the size is 1. For groups, the size is the number of columns in the group.
#[derive(Debug, Eq, PartialEq)]
pub struct TraceCols {
    name: Identifier,
    size: u64,
}

impl TraceCols {
    pub fn new(name: Identifier, size: u64) -> Self {
        Self { name, size }
    }

    pub fn name(&self) -> &str {
        self.name.name()
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

/// [TraceAccess] is used to indicate a column in the trace by specifying its index within a set of
/// [TraceCols] with the given identifier. If the identifier refers to a single column ([TraceCols]
/// with size 1), then the index is always zero.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TraceAccess {
    name: Identifier,
    idx: usize,
}

impl TraceAccess {
    pub fn new(name: Identifier, idx: usize) -> Self {
        Self { name, idx }
    }

    pub fn name(&self) -> &str {
        self.name.name()
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}

// SHARED ATOMIC TYPES
// ================================================================================================

/// [Identifier] is used to represent variable names.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct Identifier(pub String);

impl Identifier {
    /// Returns the name of the identifier.
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

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
