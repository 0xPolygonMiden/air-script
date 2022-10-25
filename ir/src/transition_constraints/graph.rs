use super::TraceColumns;
use crate::error::SemanticError;
use parser::ast::{self, Identifier, TransitionExpr};

// ALGEBRAIC GRAPH
// ================================================================================================

/// The AlgebraicGraph is a directed acyclic graph used to represent transition constraints. To
/// store it compactly, it is represented as a vector of nodes where each node references other
/// nodes by their index in the vector.
///
/// Within the graph, constraint expressions can overlap and share subgraphs, since new expressions
/// reuse matching existing nodes when they are added, rather than creating new nodes.
///
/// - Leaf nodes (with no outgoing edges) are constants or references to trace cells (i.e. column 0
///   in the current row or column 5 in the next row).
/// - Tip nodes with no incoming edges (no parent nodes) always represent constraints, although they
///   do not necessarily represent all constraints. There could be constraints which are also
///   subgraphs of other constraints.
#[derive(Default, Debug)]
pub struct AlgebraicGraph {
    /// All nodes in the graph.
    nodes: Vec<Node>,
}

impl AlgebraicGraph {
    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    /// Returns the node with the specified index.
    pub fn node(&self, index: &NodeIndex) -> &Node {
        &self.nodes[index.0]
    }

    /// Returns the degree of the subgraph which has the specified node as its tip.
    pub fn degree(&self, index: &NodeIndex) -> u8 {
        // recursively walk the subgraph and compute the degree from the operation and child nodes
        match self.node(index).op() {
            Operation::Const(_)
            | Operation::MainTraceCurrentRow(_)
            | Operation::MainTraceNextRow(_) => 1_u8,
            Operation::Neg(index) => self.degree(index),
            Operation::Add(lhs, rhs) => std::cmp::max(self.degree(lhs), self.degree(rhs)),
            Operation::Mul(lhs, rhs) => self.degree(lhs) + self.degree(rhs),
            Operation::Exp(index, exp) => self.degree(index) * (*exp as u8),
        }
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// Add the expression to the graph and return the result index. Expressions are added
    /// recursively to reuse existing matching nodes.
    pub(super) fn insert_expr(
        &mut self,
        expr: ast::TransitionExpr,
        trace_columns: &TraceColumns,
    ) -> Result<NodeIndex, SemanticError> {
        match expr {
            TransitionExpr::Const(value) => Ok(self.insert_op(Operation::Const(value))),
            TransitionExpr::Next(Identifier(ident)) => {
                let index = trace_columns.get_column_index(&ident)?;
                // insert the next row column node.
                Ok(self.insert_op(Operation::MainTraceNextRow(index)))
            }
            TransitionExpr::Var(Identifier(ident)) => {
                // since variable definitions are not possible yet, the identifier must match one of
                // the declared trace columns.
                let index = trace_columns.get_column_index(&ident)?;
                // insert the current row column node.
                Ok(self.insert_op(Operation::MainTraceCurrentRow(index)))
            }
            TransitionExpr::Add(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(*lhs, trace_columns)?;
                let rhs = self.insert_expr(*rhs, trace_columns)?;
                // add the expression.
                Ok(self.insert_op(Operation::Add(lhs, rhs)))
            }
            TransitionExpr::Sub(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(*lhs, trace_columns)?;
                let rhs = self.insert_expr(*rhs, trace_columns)?;
                // negate the right hand side.
                let rhs = self.insert_op(Operation::Neg(rhs));
                // add the expression.
                Ok(self.insert_op(Operation::Add(lhs, rhs)))
            }
            TransitionExpr::Mul(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(*lhs, trace_columns)?;
                let rhs = self.insert_expr(*rhs, trace_columns)?;
                // add the expression.
                Ok(self.insert_op(Operation::Mul(lhs, rhs)))
            }
            TransitionExpr::Exp(lhs, rhs) => {
                // add base subexpression.
                let lhs = self.insert_expr(*lhs, trace_columns)?;
                // add exponent subexpression.
                Ok(self.insert_op(Operation::Exp(lhs, rhs as usize)))
            }
        }
    }

    /// Insert the operation and return its node index. If an identical node already exists, return
    /// that index instead.
    fn insert_op(&mut self, op: Operation) -> NodeIndex {
        self.nodes.iter().position(|n| *n.op() == op).map_or_else(
            || {
                // create a new node.
                let index = self.nodes.len();
                self.nodes.push(Node { op });
                NodeIndex(index)
            },
            |index| {
                // return the existing node's index.
                NodeIndex(index)
            },
        )
    }
}

/// Reference to a node in a graph by its index in the nodes vector of the graph struct.
#[derive(Debug, Eq, PartialEq, Default)]
pub struct NodeIndex(usize);

#[derive(Debug)]
pub struct Node {
    /// The operation represented by this node
    op: Operation,
}

impl Node {
    pub fn op(&self) -> &Operation {
        &self.op
    }
}

/// A transition constraint operation or value reference.
#[derive(Debug, Eq, PartialEq)]
pub enum Operation {
    Const(u64),
    /// An identifier for a specific column in the current row in the main trace. The inner value is
    /// the index of the column within the trace.
    MainTraceCurrentRow(usize),
    /// An identifier for a specific column in the next row in the main trace. The inner value is
    /// the index of the column within the trace.
    MainTraceNextRow(usize),
    /// Negation operation applied to the node with the specified index.
    Neg(NodeIndex),
    /// Addition operation applied to the nodes with the specified indices.
    Add(NodeIndex, NodeIndex),
    /// Multiplication operation applied to the nodes with the specified indices.
    Mul(NodeIndex, NodeIndex),
    /// Exponentiation operation applied to the node with the specified index, using the provided
    /// value as the power.
    Exp(NodeIndex, usize),
}
