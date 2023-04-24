pub(crate) use air_script_core::{
    AccessType, ComprehensionContext, ConstantBinding, ConstantValueExpr, Expression, Identifier,
    Iterable, ListComprehension, ListFolding, ListFoldingValueExpr, Range, SymbolAccess,
    TraceBinding, TraceSegment, VariableBinding, VariableValueExpr,
};

// declaration modules
pub mod evaluators;
pub use evaluators::*;

pub mod function;
pub use function::*;

pub mod periodic_columns;
pub use periodic_columns::PeriodicColumn;

pub mod pub_inputs;
pub use pub_inputs::PublicInput;

pub mod random_values;
pub use random_values::*;

// constraint modules
pub mod boundary_constraints;
pub use boundary_constraints::*;

pub mod integrity_constraints;
pub use integrity_constraints::*;

// AST
// ================================================================================================

/// [Source] is the root node of the AST representing the AIR constraints file.
#[derive(Debug, Eq, PartialEq)]
pub struct Source(pub Vec<SourceSection>);

/// Source is divided into SourceSections. Each source section is responsible for declarations of a
/// specific type or for defining constraints of a specific type.
/// - AirDef: Name of the air constraints module.
///
/// The type declaration sections are:
/// - Constant: A constant is represented by a name and a value. Each [ConstantBinding] source
///   section declares a single constant.
/// - EvaluatorFunction: Evaluator functions take descriptions of the main and auxiliary traces as
///   input, and enforce integrity constraints on those trace columns. Each [EvaluatorFunction]
///   source section declares a single evaluator function
/// - Function: Functions are represented by a name and a list of parameters and returns an
///   expression. Each [Function] source section declares a single function.
/// - PeriodicColumns: Periodic columns are each represented by a fixed-size array with all of its
///   elements specified. The array length is expected to be a power of 2, but this is not checked
///   during parsing.
/// - PublicInputs: Public inputs are each represented by a fixed-size array. At least one public
///   input is required, but there is no limit to the number of public inputs that can be specified.
/// - RandomValues: Random Values represent the randomness sent by the Verifier.
/// - Trace: A vector of trace segments, each containing a vector of trace bindings, which bind an
///   identifier to one or more columns in the execution trace.
///
/// The constraint definition sections are:
/// - BoundaryConstraints: Boundary Constraints to be enforced on the boundaries of columns defined
///   in the TraceCols section. Currently there are two types of boundaries, First and Last
///   representing the first and last rows of the column.
/// - IntegrityConstraints: Integrity Constraints to be enforced on the trace columns defined
///   in the TraceCols section.
#[derive(Debug, Eq, PartialEq)]
pub enum SourceSection {
    // AIR name definition
    AirDef(Identifier),

    // type declarations
    Constant(ConstantBinding),
    EvaluatorFunction(EvaluatorFunction),
    Function(Function),
    Trace(Vec<Vec<TraceBinding>>),
    PublicInputs(Vec<PublicInput>),
    PeriodicColumns(Vec<PeriodicColumn>),
    RandomValues(RandomValues),

    // constraint definitions
    BoundaryConstraints(Vec<BoundaryStmt>),
    IntegrityConstraints(Vec<IntegrityStmt>),
}

// TRACE
// ================================================================================================

/// Given a named trace segment and a vector of (Identifier, size) pairs, returns a vector of trace
/// bindings.
pub fn build_segment_bindings(
    trace_segment: TraceSegment,
    segment_name: Identifier,
    bindings: Vec<(Identifier, u64)>,
) -> Vec<TraceBinding> {
    let mut trace_cols = Vec::new();
    let trace_segment = trace_segment.into();

    let mut offset = 0;
    for (ident, size) in bindings.into_iter() {
        let size = size as usize;
        trace_cols.push(TraceBinding::new(ident, trace_segment, offset, size));
        offset += size;
    }

    // the size for the binding for the entire segment is the sum of the other sizes.
    let segment_binding = TraceBinding::new(segment_name, trace_segment, 0, offset);
    trace_cols.push(segment_binding);

    trace_cols
}

/// Given a matrix of (Identifier, size) pairs representing a multi-segment execution trace, returns
/// a matrix of trace bindings.
pub fn build_trace_bindings(bindings: Vec<Vec<(Identifier, u64)>>) -> Vec<Vec<TraceBinding>> {
    let mut trace_cols = Vec::new();

    for (segment, bindings) in bindings.into_iter().enumerate() {
        let mut segment_cols = Vec::new();
        let mut offset = 0;
        for (ident, size) in bindings.into_iter() {
            let size = size as usize;
            segment_cols.push(TraceBinding::new(ident, segment, offset, size));
            offset += size;
        }
        trace_cols.push(segment_cols);
    }

    trace_cols
}
