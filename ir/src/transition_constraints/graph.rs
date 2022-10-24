use super::{SemanticError, SymbolTable};
use crate::symbol_table::IdentifierType;
use parser::ast::{Identifier, TransitionExpr};

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
            Operation::Constant(_)
            | Operation::MainTraceCurrentRow(_)
            | Operation::MainTraceNextRow(_)
            // TODO: check on degree calculation for auxiliary trace rows
            | Operation::AuxTraceCurrentRow(_)
            | Operation::AuxTraceNextRow(_) 
            // TODO: check the calculation of degree for periodic columns
            | Operation::PeriodicColumn(_) => 1_u8,
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
        symbol_table: &SymbolTable,
        expr: TransitionExpr,
    ) -> Result<NodeIndex, SemanticError> {
        match expr {
            TransitionExpr::Constant(value) => Ok(self.insert_op(Operation::Constant(value))),
            TransitionExpr::Next(Identifier(ident)) => self.insert_next(symbol_table, &ident),
            TransitionExpr::Variable(Identifier(ident)) => {
                self.insert_variable(symbol_table, &ident)
            }
            TransitionExpr::Add(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(symbol_table, *lhs)?;
                let rhs = self.insert_expr(symbol_table, *rhs)?;
                // add the expression.
                Ok(self.insert_op(Operation::Add(lhs, rhs)))
            }
            TransitionExpr::Subtract(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(symbol_table, *lhs)?;
                let rhs = self.insert_expr(symbol_table, *rhs)?;
                // negate the right hand side.
                let rhs = self.insert_op(Operation::Neg(rhs));
                // add the expression.
                Ok(self.insert_op(Operation::Add(lhs, rhs)))
            }
        }
    }

    fn insert_next(
        &mut self,
        symbol_table: &SymbolTable,
        ident: &str,
    ) -> Result<NodeIndex, SemanticError> {
        let col_type = symbol_table.get_type(ident)?;

        // a "next" variable expression always references an execution trace columns
        match col_type {
            IdentifierType::MainTraceColumn(index) => {
                Ok(self.insert_op(Operation::MainTraceNextRow(index)))
            }
            IdentifierType::AuxTraceColumn(index) => Ok(self.insert_op(Operation::AuxTraceNextRow(index))),
            _ => Err(SemanticError::InvalidUsage(format!(
                "Identifier {} was declared as a {} not as a trace column",
                ident, col_type
            ))),
        }
    }

    fn insert_variable(
        &mut self,
        symbol_table: &SymbolTable,
        ident: &str,
    ) -> Result<NodeIndex, SemanticError> {
        let col_type = symbol_table.get_type(ident)?;

        // since variable definitions are not possible yet, the identifier must match one of
        // the declared trace columns or one of the declared periodic columns.
        match col_type {
            IdentifierType::MainTraceColumn(index) => {
                Ok(self.insert_op(Operation::MainTraceCurrentRow(index)))
            }
            IdentifierType::AuxTraceColumn(index) => {
                Ok(self.insert_op(Operation::AuxTraceCurrentRow(index)))
            }
            IdentifierType::PeriodicColumn(index) => {
                Ok(self.insert_op(Operation::PeriodicColumn(index)))
            }
            _ => Err(SemanticError::InvalidUsage(format!(
                "Identifier {} was declared as a {} not as a trace column",
                ident, col_type
            ))),
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
    Constant(u64),
    /// An identifier for a for a cell in the specified column in the current row in the main trace. 
    /// The inner value is the index of the column within the trace.
    MainTraceCurrentRow(usize),
    /// An identifier for a cell in the specified column in the next row in the main trace. The 
    /// inner value is the index of the column within the trace.
    MainTraceNextRow(usize),
    /// An identifier for a cell in the specified column in the current row in the auxiliary trace. 
    /// The inner value is the index of the column within the trace.
    AuxTraceCurrentRow(usize),
    /// An identifier for a cell in the specified column in the next row in the auxiliary trace. The
    /// inner value is the index of the column within the trace.
    AuxTraceNextRow(usize),
    /// An identifier for a periodic value from a specified periodic column. The inner value is the
    /// index of the periodic column within the declared periodic columns. The periodic value made 
    /// available from the specified column is based on the current row of the trace.
    PeriodicColumn(usize),
    /// Negation operation applied to the node with the specified index.
    Neg(NodeIndex),
    /// Addition operation applied to the nodes with the specified indices.
    Add(NodeIndex, NodeIndex),
    /// Multiplication operation applied to the nodes with the specified indices.
    #[allow(dead_code)]
    Mul(NodeIndex, NodeIndex),
    /// Exponentiation operation applied to the node with the specified index, using the provided
    /// value as the power.
    #[allow(dead_code)]
    Exp(NodeIndex, usize),
}
