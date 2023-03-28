pub(crate) use air_script_core::{
    ColumnGroup, ComprehensionContext, Constant, ConstantType, Expression, Identifier, Iterable,
    ListComprehension, ListFoldingType, ListFoldingValueType, MatrixAccess, Range, TraceAccess,
    TraceBinding, TraceBindingAccess, TraceBindingAccessSize, TraceSegment, Variable, VariableType,
    VectorAccess,
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
    Trace(Vec<Vec<TraceBinding>>),
    PublicInputs(Vec<PublicInput>),
    PeriodicColumns(Vec<PeriodicColumn>),
    RandomValues(RandomValues),
    EvaluatorFunction(EvaluatorFunction),
    BoundaryConstraints(Vec<BoundaryStmt>),
    IntegrityConstraints(Vec<IntegrityStmt>),
}

// TRACE
// ================================================================================================

/// Given a vector of identifiers and their trace segment, returns a vector of trace bindings.
pub fn build_trace_bindings(
    trace_segment: TraceSegment,
    bindings: Vec<(Identifier, u64)>,
) -> Vec<TraceBinding> {
    let mut trace_cols = Vec::new();

    let mut offset = 0;
    for (ident, size) in bindings.into_iter() {
        trace_cols.push(TraceBinding::new(ident, trace_segment.into(), offset, size));
        offset += size as usize;
    }

    trace_cols
}

pub fn build_column_groups(
    trace_segment: TraceSegment,
    groups: Vec<(Identifier, u64)>,
) -> Vec<ColumnGroup> {
    let mut trace_cols = Vec::new();

    for (ident, size) in groups.into_iter() {
        trace_cols.push(ColumnGroup::new(ident, trace_segment, size));
    }

    trace_cols
}
