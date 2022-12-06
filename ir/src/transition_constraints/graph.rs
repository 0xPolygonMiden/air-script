use super::{
    super::BTreeMap, degree::TransitionConstraintDegree, SemanticError, SymbolTable, TraceSegment,
};
use crate::symbol_table::IdentifierType;
use parser::ast::{
    constants::ConstantType, Identifier, MatrixAccess, TransitionExpr, VectorAccess,
};

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
    pub fn degree(&self, index: &NodeIndex) -> TransitionConstraintDegree {
        let mut cycles: BTreeMap<usize, usize> = BTreeMap::new();
        let base = self.accumulate_degree(&mut cycles, index);

        if cycles.is_empty() {
            TransitionConstraintDegree::new(base)
        } else {
            TransitionConstraintDegree::with_cycles(base, cycles.values().cloned().collect())
        }
    }

    /// Recursively accumulates the base degree and the cycle lengths of the periodic columns.
    fn accumulate_degree(&self, cycles: &mut BTreeMap<usize, usize>, index: &NodeIndex) -> usize {
        // recursively walk the subgraph and compute the degree from the operation and child nodes
        match self.node(index).op() {
            Operation::Constant(_) | Operation::RandomValue(_) => 0,
            Operation::TraceElement(_) => 1,
            Operation::PeriodicColumn(index, cycle_len) => {
                cycles.insert(*index, *cycle_len);
                0
            }
            Operation::Neg(index) => self.accumulate_degree(cycles, index),
            Operation::Add(lhs, rhs) => {
                let lhs_base = self.accumulate_degree(cycles, lhs);
                let rhs_base = self.accumulate_degree(cycles, rhs);
                lhs_base.max(rhs_base)
            }
            Operation::Sub(lhs, rhs) => {
                let lhs_base = self.accumulate_degree(cycles, lhs);
                let rhs_base = self.accumulate_degree(cycles, rhs);
                lhs_base.max(rhs_base)
            }
            Operation::Mul(lhs, rhs) => {
                let lhs_base = self.accumulate_degree(cycles, lhs);
                let rhs_base = self.accumulate_degree(cycles, rhs);
                lhs_base + rhs_base
            }
            Operation::Exp(lhs, rhs) => {
                let lhs_base = self.accumulate_degree(cycles, lhs);
                lhs_base * rhs
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
    ) -> Result<(TraceSegment, NodeIndex), SemanticError> {
        match expr {
            TransitionExpr::Const(value) => {
                // constraint target defaults to Main trace.
                let trace_segment = 0;
                let node_index = self.insert_op(Operation::Constant(ConstantValue::Inline(value)));
                Ok((trace_segment, node_index))
            }
            TransitionExpr::Elem(Identifier(ident)) => {
                self.insert_symbol_access(symbol_table, &ident)
            }
            TransitionExpr::VectorAccess(vector_access) => {
                self.insert_vector_access(symbol_table, &vector_access)
            }
            TransitionExpr::MatrixAccess(matrix_access) => {
                self.insert_matrix_access(symbol_table, &matrix_access)
            }
            TransitionExpr::Next(Identifier(ident)) => self.insert_next(symbol_table, &ident),
            TransitionExpr::Rand(index) => {
                // constraint target for random values defaults to the second trace segment.
                // TODO: make this more general, so random values from further trace segments can be
                // used. This requires having a way to describe different sets of randomness in
                // the AirScript syntax.
                let trace_segment = 1;
                let node_index = self.insert_op(Operation::RandomValue(index));
                Ok((trace_segment, node_index))
            }
            TransitionExpr::Add(lhs, rhs) => {
                // add both subexpressions.
                let (lhs_segment, lhs) = self.insert_expr(symbol_table, *lhs)?;
                let (rhs_segment, rhs) = self.insert_expr(symbol_table, *rhs)?;
                // add the expression.
                let trace_segment = lhs_segment.max(rhs_segment);
                let node_index = self.insert_op(Operation::Add(lhs, rhs));
                Ok((trace_segment, node_index))
            }
            TransitionExpr::Sub(lhs, rhs) => {
                // add both subexpressions.
                let (lhs_segment, lhs) = self.insert_expr(symbol_table, *lhs)?;
                let (rhs_segment, rhs) = self.insert_expr(symbol_table, *rhs)?;
                // add the expression.
                let trace_segment = lhs_segment.max(rhs_segment);
                let node_index = self.insert_op(Operation::Sub(lhs, rhs));
                Ok((trace_segment, node_index))
            }
            TransitionExpr::Mul(lhs, rhs) => {
                // add both subexpressions.
                let (lhs_segment, lhs) = self.insert_expr(symbol_table, *lhs)?;
                let (rhs_segment, rhs) = self.insert_expr(symbol_table, *rhs)?;
                // add the expression.
                let trace_segment = lhs_segment.max(rhs_segment);
                let node_index = self.insert_op(Operation::Mul(lhs, rhs));
                Ok((trace_segment, node_index))
            }
            TransitionExpr::Exp(lhs, rhs) => {
                // add base subexpression.
                let (trace_segment, lhs) = self.insert_expr(symbol_table, *lhs)?;
                // add exponent subexpression.
                let node_index = self.insert_op(Operation::Exp(lhs, rhs as usize));
                Ok((trace_segment, node_index))
            }
        }
    }

    fn insert_next(
        &mut self,
        symbol_table: &SymbolTable,
        ident: &str,
    ) -> Result<(TraceSegment, NodeIndex), SemanticError> {
        let col_type = symbol_table.get_type(ident)?;

        match col_type {
            IdentifierType::TraceColumn(column) => {
                let trace_segment = column.trace_segment();
                let trace_access = TraceAccess::new(trace_segment, column.col_idx(), 1);
                let node_index = self.insert_op(Operation::TraceElement(trace_access));
                Ok((trace_segment, node_index))
            }
            _ => Err(SemanticError::InvalidUsage(format!(
                "Identifier {} was declared as a {} not as a trace column",
                ident, col_type
            ))),
        }
    }

    fn insert_symbol_access(
        &mut self,
        symbol_table: &SymbolTable,
        ident: &str,
    ) -> Result<(TraceSegment, NodeIndex), SemanticError> {
        let elem_type = symbol_table.get_type(ident)?;
        match elem_type {
            IdentifierType::TraceColumn(column) => {
                let trace_segment = column.trace_segment();
                let trace_access = TraceAccess::new(trace_segment, column.col_idx(), 0);
                let node_index = self.insert_op(Operation::TraceElement(trace_access));
                Ok((trace_segment, node_index))
            }
            IdentifierType::PeriodicColumn(index, cycle_len) => {
                // constraint target defaults to Main trace.
                let trace_segment = 0;
                let node_index = self.insert_op(Operation::PeriodicColumn(*index, *cycle_len));
                Ok((trace_segment, node_index))
            }
            IdentifierType::Constant(ConstantType::Scalar(_)) => {
                let trace_segment = 0;
                let node_index = self.insert_op(Operation::Constant(ConstantValue::Scalar(
                    ident.to_string(),
                )));
                Ok((trace_segment, node_index))
            }
            _ => Err(SemanticError::InvalidUsage(format!(
                "Identifier {} was declared as a {} which is not a supported type.",
                ident, elem_type
            ))),
        }
    }

    /// Validates and adds a vector access to the graph.
    /// Returns an error if the identifier's value is not of a supported type.
    fn insert_vector_access(
        &mut self,
        symbol_table: &SymbolTable,
        vector_access: &VectorAccess,
    ) -> Result<(TraceSegment, NodeIndex), SemanticError> {
        let symbol_type = symbol_table.access_vector_element(vector_access)?;
        if !matches!(
            symbol_type,
            IdentifierType::Constant(ConstantType::Vector(_))
        ) {
            return Err(SemanticError::invalid_vector_access(
                vector_access,
                symbol_type,
            ));
        }

        let trace_segment = 0;
        let node_index = self.insert_op(Operation::Constant(ConstantValue::Vector(
            vector_access.clone(),
        )));
        Ok((trace_segment, node_index))
    }

    /// Validates and adds a matrix access to the graph.
    /// Returns an error if the identifier's value is not of a supported type.
    fn insert_matrix_access(
        &mut self,
        symbol_table: &SymbolTable,
        matrix_access: &MatrixAccess,
    ) -> Result<(TraceSegment, NodeIndex), SemanticError> {
        let symbol_type = symbol_table.access_matrix_element(matrix_access)?;
        if !matches!(
            symbol_type,
            IdentifierType::Constant(ConstantType::Matrix(_))
        ) {
            return Err(SemanticError::invalid_matrix_access(
                matrix_access,
                symbol_type,
            ));
        }

        let trace_segment = 0;
        let node_index = self.insert_op(Operation::Constant(ConstantValue::Matrix(
            matrix_access.clone(),
        )));
        Ok((trace_segment, node_index))
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
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
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
    /// An inlined or named constant with identifier and access indices.
    Constant(ConstantValue),
    /// An identifier for an element in the trace segment, column, and row offset specified by the
    /// [TraceAccess]
    TraceElement(TraceAccess),
    /// An identifier for a periodic value from a specified periodic column. The first inner value
    /// is the index of the periodic column within the declared periodic columns. The second inner
    /// value is the length of the column's periodic cycle. The periodic value made available from
    /// the specified column is based on the current row of the trace.
    PeriodicColumn(usize, usize),
    /// A random value provided by the verifier. The inner value is the index of this random value
    /// in the array of all random values.
    RandomValue(usize),
    /// Negation operation applied to the node with the specified index.
    Neg(NodeIndex),
    /// Addition operation applied to the nodes with the specified indices.
    Add(NodeIndex, NodeIndex),
    /// Subtraction operation applied to the nodes with the specified indices.
    Sub(NodeIndex, NodeIndex),
    /// Multiplication operation applied to the nodes with the specified indices.
    Mul(NodeIndex, NodeIndex),
    /// Exponentiation operation applied to the node with the specified index, using the provided
    /// value as the power.
    Exp(NodeIndex, usize),
}

/// Access information for getting an element in the execution trace. The trace_segment specifies
/// how many trace commitments have preceded the specified segment. `col_idx` specifies the index
/// of the column within that trace segment, and `row_offset` specifies the offset from the current
/// row. For example, an element in the "next" row of the "main" trace would be specified by
/// a trace_segment of 0 and a row_offset of 1.
#[derive(Debug, Eq, PartialEq)]
pub struct TraceAccess {
    trace_segment: TraceSegment,
    col_idx: usize,
    row_offset: usize,
}

impl TraceAccess {
    fn new(trace_segment: TraceSegment, col_idx: usize, row_offset: usize) -> Self {
        Self {
            trace_segment,
            col_idx,
            row_offset,
        }
    }

    /// Gets the column index of this [TraceAccess].
    pub fn col_idx(&self) -> usize {
        self.col_idx
    }

    /// Gets the row offset of this [TraceAccess].
    pub fn row_offset(&self) -> usize {
        self.row_offset
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ConstantValue {
    Inline(u64),
    Scalar(String),
    Vector(VectorAccess),
    Matrix(MatrixAccess),
}
