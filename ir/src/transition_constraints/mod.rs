use super::{SemanticError, SymbolTable};
use parser::ast;

mod graph;
pub use graph::{AlgebraicGraph, NodeIndex, Operation};

// TRANSITION CONSTRAINTS
// ================================================================================================

#[derive(PartialEq)]
enum ConstraintType {
    Main,
    Auxiliary,
}

#[derive(Default, Debug)]
pub(super) struct TransitionConstraints {
    /// The indices of the entry nodes for each of the transition constraints against the main trace
    /// in the graph.
    main_constraints: Vec<NodeIndex>,

    /// The indices of the entry nodes for each of the transition constraints against the auxiliary
    /// trace in the graph.
    aux_constraints: Vec<NodeIndex>,

    /// A directed acyclic graph which represents all of the transition constraints.
    graph: AlgebraicGraph,
}

impl TransitionConstraints {
    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    /// Returns a vector of the degrees of the transition constraints.
    pub fn main_degrees(&self) -> Vec<u8> {
        self.main_constraints
            .iter()
            .map(|entry_index| self.graph.degree(entry_index))
            .collect()
    }

    /// Returns all transition constraints against the main execution trace as a vector of
    /// [NodeIndex] where each index is the tip of the subgraph representing the constraint within
    /// the [AlgebraicGraph].
    pub fn main_constraints(&self) -> &[NodeIndex] {
        &self.main_constraints
    }

    /// Returns a vector of the degrees of the transition constraints.
    pub fn aux_degrees(&self) -> Vec<u8> {
        self.aux_constraints
            .iter()
            .map(|entry_index| self.graph.degree(entry_index))
            .collect()
    }

    /// Returns all transition constraints against the auxiliary execution trace as a vector of
    /// [NodeIndex] where each index is the tip of the subgraph representing the constraint within
    /// the [AlgebraicGraph].
    pub fn aux_constraints(&self) -> &[NodeIndex] {
        &self.aux_constraints
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
        let (constraint_type, entry_index) = self.graph.insert_expr(symbol_table, expr)?;

        // add the transition constraint.
        match constraint_type {
            ConstraintType::Main => self.main_constraints.push(entry_index),
            ConstraintType::Auxiliary => self.aux_constraints.push(entry_index),
        }

        Ok(())
    }
}
