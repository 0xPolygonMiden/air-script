use super::{degree::TransitionConstraintDegree, ConstraintType, SemanticError, SymbolTable};
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
    pub fn degree(&self, cycle_lens: &[usize], index: &NodeIndex) -> TransitionConstraintDegree {
        let (base, mut periodic_columns) = self.accumulate_degree(index);

        if periodic_columns.is_empty() {
            TransitionConstraintDegree::new(base)
        } else {
            periodic_columns.sort();
            periodic_columns.dedup();
            let mut cycles = Vec::with_capacity(periodic_columns.len());
            for index in periodic_columns.iter() {
                cycles.push(cycle_lens[*index]);
            }

            TransitionConstraintDegree::with_cycles(base, cycles)
        }
    }

    /// TODO
    fn accumulate_degree(&self, index: &NodeIndex) -> (usize, Vec<usize>) {
        // recursively walk the subgraph and compute the degree from the operation and child nodes
        match self.node(index).op() {
            Operation::Constant(_) | Operation::RandomValue(_) => (0, vec![]),
            Operation::MainTraceCurrentRow(_)
            | Operation::MainTraceNextRow(_)
            | Operation::AuxTraceCurrentRow(_)
            | Operation::AuxTraceNextRow(_) => (1, vec![]),
            Operation::PeriodicColumn(index) => (0, vec![*index]),
            Operation::Neg(index) => self.accumulate_degree(index),
            Operation::Add(lhs, rhs) => {
                let (lhs_base, mut columns) = self.accumulate_degree(lhs);
                let (rhs_base, mut rhs_columns) = self.accumulate_degree(rhs);
                let base = lhs_base.max(rhs_base);
                columns.append(&mut rhs_columns);
                (base, columns)
            }
            Operation::Mul(lhs, rhs) => {
                let (lhs_base, mut columns) = self.accumulate_degree(lhs);
                let (rhs_base, mut rhs_columns) = self.accumulate_degree(rhs);
                let base = lhs_base + rhs_base;
                columns.append(&mut rhs_columns);
                (base, columns)
            }
        }
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// Add the expression to the graph and return the result index and a constraint type indicating
    /// whether it is applied to the main execution trace or an auxiliary trace. Expressions are
    /// added recursively to reuse existing matching nodes.
    pub(super) fn insert_expr(
        &mut self,
        symbol_table: &SymbolTable,
        expr: TransitionExpr,
    ) -> Result<(ConstraintType, NodeIndex), SemanticError> {
        match expr {
            TransitionExpr::Constant(value) => {
                // constraint target defaults to Main trace.
                let constraint_type = ConstraintType::Main;
                let node_index = self.insert_op(Operation::Constant(value));
                Ok((constraint_type, node_index))
            }
            TransitionExpr::Variable(Identifier(ident)) => {
                self.insert_variable(symbol_table, &ident)
            }
            TransitionExpr::Next(Identifier(ident)) => self.insert_next(symbol_table, &ident),
            TransitionExpr::RandomValue(index) => {
                let constraint_type = ConstraintType::Auxiliary;
                let node_index = self.insert_op(Operation::RandomValue(index));
                Ok((constraint_type, node_index))
            }
            TransitionExpr::Add(lhs, rhs) => {
                // add both subexpressions.
                let (lhs_type, lhs) = self.insert_expr(symbol_table, *lhs)?;
                let (rhs_type, rhs) = self.insert_expr(symbol_table, *rhs)?;
                // add the expression.
                let constraint_type = get_binop_constraint_type(lhs_type, rhs_type);
                let node_index = self.insert_op(Operation::Add(lhs, rhs));
                Ok((constraint_type, node_index))
            }
            TransitionExpr::Subtract(lhs, rhs) => {
                // add both subexpressions.
                let (lhs_type, lhs) = self.insert_expr(symbol_table, *lhs)?;
                let (rhs_type, rhs) = self.insert_expr(symbol_table, *rhs)?;
                // negate the right hand side.
                let rhs = self.insert_op(Operation::Neg(rhs));
                // add the expression.
                let constraint_type = get_binop_constraint_type(lhs_type, rhs_type);
                let node_index = self.insert_op(Operation::Add(lhs, rhs));
                Ok((constraint_type, node_index))
            }
        }
    }

    fn insert_next(
        &mut self,
        symbol_table: &SymbolTable,
        ident: &str,
    ) -> Result<(ConstraintType, NodeIndex), SemanticError> {
        let col_type = symbol_table.get_type(ident)?;

        // a "next" variable expression always references an execution trace columns
        match col_type {
            IdentifierType::MainTraceColumn(index) => {
                let constraint_type = ConstraintType::Main;
                let node_index = self.insert_op(Operation::MainTraceNextRow(index));
                Ok((constraint_type, node_index))
            }
            IdentifierType::AuxTraceColumn(index) => {
                let constraint_type = ConstraintType::Auxiliary;
                let node_index = self.insert_op(Operation::AuxTraceNextRow(index));
                Ok((constraint_type, node_index))
            }
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
    ) -> Result<(ConstraintType, NodeIndex), SemanticError> {
        let col_type = symbol_table.get_type(ident)?;

        // since variable definitions are not possible yet, the identifier must match one of
        // the declared trace columns or one of the declared periodic columns.
        match col_type {
            IdentifierType::MainTraceColumn(index) => {
                let constraint_type = ConstraintType::Main;
                let node_index = self.insert_op(Operation::MainTraceCurrentRow(index));
                Ok((constraint_type, node_index))
            }
            IdentifierType::AuxTraceColumn(index) => {
                let constraint_type = ConstraintType::Auxiliary;
                let node_index = self.insert_op(Operation::AuxTraceCurrentRow(index));
                Ok((constraint_type, node_index))
            }
            IdentifierType::PeriodicColumn(index) => {
                // constraint target defaults to Main trace.
                let constraint_type = ConstraintType::Main;
                let node_index = self.insert_op(Operation::PeriodicColumn(index));
                Ok((constraint_type, node_index))
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

fn get_binop_constraint_type(lhs_type: ConstraintType, rhs_type: ConstraintType) -> ConstraintType {
    if lhs_type == ConstraintType::Auxiliary || rhs_type == ConstraintType::Auxiliary {
        ConstraintType::Auxiliary
    } else {
        ConstraintType::Main
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
    /// A random value provided by the verifier. The inner value is the index of this random value
    /// in the array of all random values.
    RandomValue(usize),
    /// Negation operation applied to the node with the specified index.
    Neg(NodeIndex),
    /// Addition operation applied to the nodes with the specified indices.
    Add(NodeIndex, NodeIndex),
    /// Multiplication operation applied to the nodes with the specified indices.
    #[allow(dead_code)]
    Mul(NodeIndex, NodeIndex),
}
