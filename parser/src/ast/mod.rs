pub(crate) use air_script_core::{
    ComprehensionContext, Constant, ConstantType, Expression, Identifier, IndexedTraceAccess,
    Iterable, ListComprehension, ListFoldingType, ListFoldingValueType, MatrixAccess,
    NamedTraceAccess, Range, Variable, VariableType, VectorAccess,
};

pub mod pub_inputs;
pub use pub_inputs::PublicInput;

pub mod periodic_columns;
pub use periodic_columns::PeriodicColumn;

pub mod boundary_constraints;
pub use boundary_constraints::*;

pub mod integrity_constraints;
pub use integrity_constraints::*;

pub mod random_values;
pub use random_values::*;

pub mod evaluator_function;
pub use evaluator_function::*;

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
/// - RandomValues: Random Values represent the randomness sent by the Verifier.
/// - EvaluatorFunction: Evaluator Functions take descriptions of the main and auxiliary traces as
///   input, and enforce integrity constraints on those trace columns.
/// - BoundaryConstraints: Boundary Constraints to be enforced on the boundaries of columns defined
///   in the TraceCols section. Currently there are two types of boundaries, First and Last
///   representing the first and last rows of the column.
/// - IntegrityConstraints: Integrity Constraints to be enforced on the trace columns defined
///   in the TraceCols section.
#[derive(Debug, Eq, PartialEq)]
pub enum SourceSection {
    AirDef(Identifier),
    Constant(Constant),
    Trace(Trace),
    PublicInputs(Vec<PublicInput>),
    PeriodicColumns(Vec<PeriodicColumn>),
    RandomValues(RandomValues),
    EvaluatorFunction(EvaluatorFunction),
    BoundaryConstraints(Vec<BoundaryStmt>),
    IntegrityConstraints(Vec<IntegrityStmt>),
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
