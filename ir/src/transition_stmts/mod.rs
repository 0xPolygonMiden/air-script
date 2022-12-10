use std::collections::BTreeMap;

use super::{SemanticError, SymbolTable, TraceSegment};
use parser::ast::{
    Identifier, MatrixAccess, TransitionExpr, TransitionStmt, TransitionVariable,
    TransitionVariableType, VectorAccess,
};

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

    /// A vector containing all the variables defined in the transition constraints section.
    variables: Vec<TransitionVariable>,

    /// Variable roots for all the variables in the variables graph. For each element in a vector
    /// or a matrix, a new root is added with a key equal to the [VariableValue] of the element.
    variable_roots: BTreeMap<VariableValue, NodeIndex>,

    /// A directed acyclic graph which represents all of the variables defined in the transition
    /// constraints section.
    variables_graph: AlgebraicGraph,
}

impl TransitionStmts {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    pub fn new(num_trace_segments: usize) -> Self {
        Self {
            constraint_roots: vec![Vec::new(); num_trace_segments],
            constraints_graph: AlgebraicGraph::default(),
            variables: Vec::new(),
            variable_roots: BTreeMap::new(),
            variables_graph: AlgebraicGraph::default(),
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

    /// Returns all the variables defined in the transition constraints section.
    pub fn variables(&self) -> &Vec<TransitionVariable> {
        &self.variables
    }

    /// Returns variable roots map for the variables defined in the transition constraints section.
    /// The value of the map contains the tip of the subgraph representing the variable within the
    /// variables [AlgebraicGraph].
    pub fn variable_roots(&self) -> &BTreeMap<VariableValue, NodeIndex> {
        &self.variable_roots
    }

    /// Returns the [AlgebraicGraph] representing all variables defined in the transition
    /// constraints section.
    pub fn variables_graph(&self) -> &AlgebraicGraph {
        &self.variables_graph
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// Adds the provided parsed transition statement to the graph. The statement can either be a
    /// variable defined in the transition constraints section or a transition constraint.
    ///
    /// In case the statement is a variable, it is turned into a subgraph which is added to the
    /// variables [AlgebraicGraph]. The index of its entry node is then saved in the
    /// variable_roots map.
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
            TransitionStmt::Variable(variable) => {
                symbol_table.insert_transition_variable(variable)?;
                match variable.value() {
                    TransitionVariableType::Scalar(expr) => {
                        let variable_value = VariableValue::Scalar(variable.name().to_string());
                        self.insert_variable_expr(symbol_table, variable_value, expr)?;
                    }
                    TransitionVariableType::Vector(vector) => {
                        for (idx, expr) in vector.iter().enumerate() {
                            let variable_value = VariableValue::Vector(VectorAccess::new(
                                Identifier(variable.name().to_string()),
                                idx,
                            ));
                            self.insert_variable_expr(symbol_table, variable_value, expr)?;
                        }
                    }
                    TransitionVariableType::Matrix(matrix) => {
                        for (row_idx, row) in matrix.iter().enumerate() {
                            for (col_idx, expr) in row.iter().enumerate() {
                                let variable_value = VariableValue::Matrix(MatrixAccess::new(
                                    Identifier(variable.name().to_string()),
                                    row_idx,
                                    col_idx,
                                ));
                                self.insert_variable_expr(symbol_table, variable_value, expr)?;
                            }
                        }
                    }
                }

                self.variables.push(variable.clone())
            }
            TransitionStmt::Constraint(constraint) => {
                let expr = constraint.expr();

                // add it to the transition constraints graph and get its entry index.
                let (trace_segment, root_index) =
                    self.constraints_graph.insert_expr(symbol_table, expr)?;

                // the constraint should not be against an undeclared trace segment.
                if symbol_table.num_trace_segments() <= trace_segment.into() {
                    return Err(SemanticError::InvalidConstraint(
                        "Constraint against undeclared trace segment".to_string(),
                    ));
                }

                // add the transition constraint to the appropriate set of constraints.
                self.constraint_roots[trace_segment as usize].push(root_index);
            }
        }

        Ok(())
    }

    /// A helper function to insert variable expression in the variables graph as a subgraph and
    /// add its root to the variable_roots map.
    fn insert_variable_expr(
        &mut self,
        symbol_table: &mut SymbolTable,
        variable_value: VariableValue,
        expr: &TransitionExpr,
    ) -> Result<(), SemanticError> {
        // add it to the transition constraints graph and get its entry index.
        let (_, root_index) = self
            .variables_graph
            .insert_expr(symbol_table, expr.clone())?;

        self.variable_roots.insert(variable_value, root_index);
        Ok(())
    }
}
