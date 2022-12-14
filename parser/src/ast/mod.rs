use std::fmt;

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
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
pub enum SourceSection {
    AirDef(Identifier),
    TraceCols(TraceCols),
    PublicInputs(Vec<PublicInput>),
    PeriodicColumns(Vec<PeriodicColumn>),
    BoundaryConstraints(BoundaryConstraints),
    TransitionConstraints(TransitionConstraints),
}

// TRACE
// ================================================================================================

/// [TraceCols] contains the main and auxiliary trace columns of the execution trace.
#[derive(Debug, Eq, PartialEq)]
pub struct TraceCols {
    pub main_cols: Vec<Identifier>,
    pub aux_cols: Vec<Identifier>,
}

// SHARED ATOMIC TYPES
// ================================================================================================

/// [Identifier] is used to represent variable names.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct Identifier(pub String);

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}
