use super::{SemanticError, SymbolTable, TraceSegment};
use parser::ast;

mod degree;
pub use degree::TransitionConstraintDegree;

mod graph;
pub use graph::{AlgebraicGraph, ConstantValue, NodeIndex, Operation};

// CONSTANTS
// ================================================================================================

pub const MIN_CYCLE_LENGTH: usize = 2;

// TRANSITION CONSTRAINTS
// ================================================================================================

#[derive(Default, Debug)]
pub(super) struct TransitionConstraints {
    /// Transition constraints against the execution trace, where each index contains a vector of
    /// the constraint roots for all constraints against that segment of the trace. For example,
    /// constraints against the main execution trace, which is trace segment 0, will be specified by
    /// a vector in constraint_roots[0] containing a [NodeIndex] in the graph for each constraint
    /// against the main trace.
    constraint_roots: Vec<Vec<NodeIndex>>,

    /// A directed acyclic graph which represents all of the transition constraints.
    graph: AlgebraicGraph,
}

impl TransitionConstraints {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    pub fn new(num_trace_segments: usize) -> Self {
        Self {
            constraint_roots: vec![Vec::new(); num_trace_segments],
            graph: AlgebraicGraph::default(),
        }
    }

    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    /// Returns a vector of the degrees of the transition constraints for the specified trace
    /// segment.
    pub fn constraint_degrees(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<TransitionConstraintDegree> {
        if self.constraint_roots.len() <= trace_segment.into() {
            return Vec::new();
        }

        self.constraint_roots[trace_segment as usize]
            .iter()
            .map(|entry_index| self.graph.degree(entry_index))
            .collect()
    }

    /// Returns all transition constraints against the specified trace segment as a vector of
    /// [NodeIndex] where each index is the tip of the subgraph representing the constraint within
    /// the [AlgebraicGraph].
    pub fn constraints(&self, trace_segment: TraceSegment) -> &[NodeIndex] {
        if self.constraint_roots.len() <= trace_segment.into() {
            return &[];
        }

        &self.constraint_roots[trace_segment as usize]
    }

    /// Returns the [AlgebraicGraph] representing all transition constraints.
    pub fn graph(&self) -> &AlgebraicGraph {
        &self.graph
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// Adds the provided parsed transition constraint to the graph.
    ///
    /// The constraint is turned into a subgraph which is added to the [AlgebraicGraph] (reusing any
    /// existing nodes). The index of its entry node is then saved in the constraints array.
    pub(super) fn insert(
        &mut self,
        symbol_table: &SymbolTable,
        constraint: &ast::TransitionConstraint,
    ) -> Result<(), SemanticError> {
        let expr = constraint.expr();

        // add it to the transition constraints graph and get its entry index.
        let (trace_segment, root_index) = self.graph.insert_expr(symbol_table, expr)?;

        // the constraint should not be against an undeclared trace segment.
        if symbol_table.num_trace_segments() <= trace_segment.into() {
            return Err(SemanticError::InvalidConstraint(
                "Constraint against undeclared trace segment".to_string(),
            ));
        }

        // add the transition constraint to the appropriate set of constraints.
        self.constraint_roots[trace_segment as usize].push(root_index);

        Ok(())
    }
}
