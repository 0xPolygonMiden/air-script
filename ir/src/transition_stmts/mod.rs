use super::{SemanticError, SymbolTable, TraceSegment, VariableRoots};
use parser::ast::TransitionStmt;
use std::collections::BTreeMap;

mod degree;
pub use degree::TransitionConstraintDegree;

mod graph;
pub use graph::{AlgebraicGraph, ConstantValue, NodeIndex, Operation, VariableValue};

// CONSTANTS
// ================================================================================================

pub const MIN_CYCLE_LENGTH: usize = 2;

// TRANSITION CONSTRAINTS
// ================================================================================================

#[derive(Default, Debug)]
pub(super) struct TransitionStmts {
    /// Transition constraints against the execution trace, where each index contains a vector of
    /// the constraint roots for all constraints against that segment of the trace. For example,
    /// constraints against the main execution trace, which is trace segment 0, will be specified by
    /// a vector in constraint_roots[0] containing a [NodeIndex] in the graph for each constraint
    /// against the main trace.
    constraint_roots: Vec<Vec<NodeIndex>>,

    /// A directed acyclic graph which represents all of the transition constraints.
    constraints_graph: AlgebraicGraph,

    /// Variable roots for the variables used in transition constraints. For each element in a
    /// vector or a matrix, a new root is added with a key equal to the [VariableValue] of the
    /// element.
    variable_roots: VariableRoots,
}

impl TransitionStmts {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    pub fn new(num_trace_segments: usize) -> Self {
        Self {
            constraint_roots: vec![Vec::new(); num_trace_segments],
            constraints_graph: AlgebraicGraph::default(),
            variable_roots: BTreeMap::new(),
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
            .map(|entry_index| self.constraints_graph.degree(entry_index))
            .collect()
    }

    /// Returns all transition constraints against the specified trace segment as a vector of
    /// [NodeIndex] where each index is the tip of the subgraph representing the constraint within
    /// the constraints [AlgebraicGraph].
    pub fn constraints(&self, trace_segment: TraceSegment) -> &[NodeIndex] {
        if self.constraint_roots.len() <= trace_segment.into() {
            return &[];
        }

        &self.constraint_roots[trace_segment as usize]
    }

    /// Returns the [AlgebraicGraph] representing all transition constraints.
    pub fn graph(&self) -> &AlgebraicGraph {
        &self.constraints_graph
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// Adds the provided parsed transition statement to the graph. The statement can either be a
    /// variable defined in the transition constraints section or a transition constraint.
    ///
    /// In case the statement is a variable, it is added to the symbol table.
    ///
    /// In case the statement is a constraint, the constraint is turned into a subgraph which is
    /// added to the [AlgebraicGraph] (reusing any existing nodes). The index of its entry node
    /// is then saved in the constraint_roots matrix.
    pub(super) fn insert(
        &mut self,
        symbol_table: &mut SymbolTable,
        stmt: &TransitionStmt,
    ) -> Result<(), SemanticError> {
        match stmt {
            TransitionStmt::Constraint(constraint) => {
                let expr = constraint.expr();

                // add it to the transition constraints graph and get its entry index.
                let (trace_segment, root_index) = self.constraints_graph.insert_expr(
                    symbol_table,
                    expr,
                    &mut self.variable_roots,
                )?;

                // the constraint should not be against an undeclared trace segment.
                if symbol_table.num_trace_segments() <= trace_segment.into() {
                    return Err(SemanticError::InvalidConstraint(
                        "Constraint against undeclared trace segment".to_string(),
                    ));
                }

                // add the transition constraint to the appropriate set of constraints.
                self.constraint_roots[trace_segment as usize].push(root_index);
            }
            TransitionStmt::Variable(variable) => {
                symbol_table.insert_transition_variable(variable)?
            }
        }

        Ok(())
    }
}
