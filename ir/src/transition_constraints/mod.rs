use super::TraceColumns;
use crate::error::SemanticError;
use parser::ast;

mod graph;
pub use graph::{AlgebraicGraph, NodeIndex, Operation};

// TRANSITION CONSTRAINTS
// ================================================================================================

#[derive(Default, Debug)]
pub struct TransitionConstraints {
    /// The indices of the entry nodes for each of the transition constraints in the graph.
    constraints: Vec<NodeIndex>,

    /// A directed acyclic graph which represents all of the transition constraints.
    graph: AlgebraicGraph,
}

impl TransitionConstraints {
    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    /// Returns a vector of the degrees of the transition contraints.
    pub fn degrees(&self) -> Vec<u8> {
        self.constraints
            .iter()
            .map(|entry_index| self.graph.degree(entry_index))
            .collect()
    }

    /// Returns all transition constraints as a vector of [NodeIndex] where each index is the tip of
    /// the subgraph representing the constraint within the [AlgebraicGraph].
    pub fn constraints(&self) -> &[NodeIndex] {
        &self.constraints
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
        constraint: &ast::TransitionConstraint,
        trace_columns: &TraceColumns,
    ) -> Result<(), SemanticError> {
        let expr = constraint.expr();

        // add it to the transition constraints graph and get its entry index.
        let entry_index = self.graph.insert_expr(expr, trace_columns)?;

        // add the transition constraint.
        self.constraints.push(entry_index);

        Ok(())
    }
}
