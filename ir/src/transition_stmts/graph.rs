use super::{
    super::BTreeMap, degree::TransitionConstraintDegree, SemanticError, SymbolTable, TraceSegment,
};
use crate::{symbol_table::IdentifierType, VariableRoots, CURRENT_ROW, NEXT_ROW};
use parser::ast::{
    self, constants::ConstantType, Identifier, MatrixAccess, TransitionExpr,
    TransitionVariableType, VectorAccess,
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

    /// Adds the expression to the graph and returns the result index and a constraint type
    /// indicating whether it is applied to the main execution trace or an auxiliary trace.
    /// Expressions are added recursively to reuse existing matching nodes.
    pub(super) fn insert_expr(
        &mut self,
        symbol_table: &SymbolTable,
        expr: TransitionExpr,
        variable_roots: &mut VariableRoots,
    ) -> Result<(TraceSegment, NodeIndex), SemanticError> {
        match expr {
            TransitionExpr::Const(value) => {
                // constraint target defaults to Main trace.
                let trace_segment = 0;
                let node_index = self.insert_op(Operation::Constant(ConstantValue::Inline(value)));
                Ok((trace_segment, node_index))
            }
            TransitionExpr::Elem(Identifier(ident)) => {
                self.insert_symbol_access(symbol_table, &ident, variable_roots)
            }
            TransitionExpr::VectorAccess(vector_access) => {
                self.insert_vector_access(symbol_table, &vector_access, variable_roots)
            }
            TransitionExpr::MatrixAccess(matrix_access) => {
                self.insert_matrix_access(symbol_table, &matrix_access, variable_roots)
            }
            TransitionExpr::Next(trace_access) => self.insert_next(symbol_table, &trace_access),
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
                let (lhs_segment, lhs) = self.insert_expr(symbol_table, *lhs, variable_roots)?;
                let (rhs_segment, rhs) = self.insert_expr(symbol_table, *rhs, variable_roots)?;
                // add the expression.
                let trace_segment = lhs_segment.max(rhs_segment);
                let node_index = self.insert_op(Operation::Add(lhs, rhs));
                Ok((trace_segment, node_index))
            }
            TransitionExpr::Sub(lhs, rhs) => {
                // add both subexpressions.
                let (lhs_segment, lhs) = self.insert_expr(symbol_table, *lhs, variable_roots)?;
                let (rhs_segment, rhs) = self.insert_expr(symbol_table, *rhs, variable_roots)?;
                // add the expression.
                let trace_segment = lhs_segment.max(rhs_segment);
                let node_index = self.insert_op(Operation::Sub(lhs, rhs));
                Ok((trace_segment, node_index))
            }
            TransitionExpr::Mul(lhs, rhs) => {
                // add both subexpressions.
                let (lhs_segment, lhs) = self.insert_expr(symbol_table, *lhs, variable_roots)?;
                let (rhs_segment, rhs) = self.insert_expr(symbol_table, *rhs, variable_roots)?;
                // add the expression.
                let trace_segment = lhs_segment.max(rhs_segment);
                let node_index = self.insert_op(Operation::Mul(lhs, rhs));
                Ok((trace_segment, node_index))
            }
            TransitionExpr::Exp(lhs, rhs) => {
                // add base subexpression.
                let (trace_segment, lhs) = self.insert_expr(symbol_table, *lhs, variable_roots)?;
                // add exponent subexpression.
                let node_index = self.insert_op(Operation::Exp(lhs, rhs as usize));
                Ok((trace_segment, node_index))
            }
        }
    }

    /// Adds a next row operator to the graph and returns the node index and the trace segment.
    ///
    /// # Errors
    /// Returns an error if the operator is not applied to a trace column in the symbol table.
    fn insert_next(
        &mut self,
        symbol_table: &SymbolTable,
        trace_access: &ast::TraceAccess,
    ) -> Result<(TraceSegment, NodeIndex), SemanticError> {
        let col_type = symbol_table.get_type(trace_access.name())?;
        match col_type {
            IdentifierType::TraceColumns(columns) => {
                let trace_segment = columns.trace_segment();
                let trace_access = TraceAccess::new(
                    trace_segment,
                    columns.offset() + trace_access.idx(),
                    NEXT_ROW,
                );
                let node_index = self.insert_op(Operation::TraceElement(trace_access));
                Ok((trace_segment, node_index))
            }
            _ => Err(SemanticError::InvalidUsage(format!(
                "Identifier {} was declared as a {} not as a trace column",
                trace_access.name(),
                col_type
            ))),
        }
    }

    /// Adds a trace column, periodic column, named constant or a variable to the graph and returns
    /// the node index and trace segment.
    ///
    /// # Errors
    /// Returns an error if the identifier is not present in the symbol table or is not a supported
    /// type.
    fn insert_symbol_access(
        &mut self,
        symbol_table: &SymbolTable,
        ident: &str,
        variable_roots: &mut VariableRoots,
    ) -> Result<(TraceSegment, NodeIndex), SemanticError> {
        let elem_type = symbol_table.get_type(ident)?;
        match elem_type {
            IdentifierType::TraceColumns(columns) => {
                let trace_segment = columns.trace_segment();
                let trace_access = TraceAccess::new(trace_segment, columns.offset(), CURRENT_ROW);
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
            IdentifierType::TransitionVariable(transition_variable) => {
                if let TransitionVariableType::Scalar(expr) = transition_variable.value() {
                    if let Some((trace_segment, node_index)) =
                        variable_roots.get(&VariableValue::Scalar(ident.to_string()))
                    {
                        Ok((*trace_segment, *node_index))
                    } else {
                        let (trace_segment, node_index) =
                            self.insert_expr(symbol_table, expr.clone(), variable_roots)?;
                        variable_roots.insert(
                            VariableValue::Scalar(ident.to_string()),
                            (trace_segment, node_index),
                        );
                        Ok((trace_segment, node_index))
                    }
                } else {
                    Err(SemanticError::InvalidUsage(format!(
                        "Identifier {} was declared as a {} which is not a supported type.",
                        ident, elem_type
                    )))
                }
            }
            _ => Err(SemanticError::InvalidUsage(format!(
                "Identifier {} was declared as a {} which is not a supported type.",
                ident, elem_type
            ))),
        }
    }

    /// Validates and adds a vector access to the graph.
    ///
    /// # Errors
    /// Returns an error if the identifier's value is not of a supported type.
    fn insert_vector_access(
        &mut self,
        symbol_table: &SymbolTable,
        vector_access: &VectorAccess,
        variable_roots: &mut VariableRoots,
    ) -> Result<(TraceSegment, NodeIndex), SemanticError> {
        let symbol_type = symbol_table.access_vector_element(vector_access)?;
        match symbol_type {
            IdentifierType::Constant(ConstantType::Vector(_)) => {
                let trace_segment = 0;
                let node_index = self.insert_op(Operation::Constant(ConstantValue::Vector(
                    vector_access.clone(),
                )));
                Ok((trace_segment, node_index))
            }
            IdentifierType::TransitionVariable(transition_variable) => {
                if let TransitionVariableType::Vector(vector) = transition_variable.value() {
                    let expr = &vector[vector_access.idx()];
                    if let Some((trace_segment, node_index)) =
                        variable_roots.get(&VariableValue::Vector(vector_access.clone()))
                    {
                        Ok((*trace_segment, *node_index))
                    } else {
                        let (trace_segment, node_index) =
                            self.insert_expr(symbol_table, expr.clone(), variable_roots)?;
                        variable_roots.insert(
                            VariableValue::Vector(vector_access.clone()),
                            (trace_segment, node_index),
                        );
                        Ok((trace_segment, node_index))
                    }
                } else {
                    Err(SemanticError::InvalidUsage(format!(
                        "Identifier {} was declared as a {} which is not a supported type.",
                        vector_access.name(),
                        symbol_type
                    )))
                }
            }
            IdentifierType::TraceColumns(columns) => {
                let trace_segment = columns.trace_segment();
                let col_idx = columns.offset() + vector_access.idx();
                let node_index = self.insert_op(Operation::TraceElement(TraceAccess::new(
                    trace_segment,
                    col_idx,
                    CURRENT_ROW,
                )));
                Ok((trace_segment, node_index))
            }
            _ => Err(SemanticError::invalid_vector_access(
                vector_access,
                symbol_type,
            )),
        }
    }

    /// Validates and adds a matrix access to the graph.
    ///
    /// # Errors
    /// Returns an error if the identifier's value is not of a supported type.
    fn insert_matrix_access(
        &mut self,
        symbol_table: &SymbolTable,
        matrix_access: &MatrixAccess,
        variable_roots: &mut VariableRoots,
    ) -> Result<(TraceSegment, NodeIndex), SemanticError> {
        let symbol_type = symbol_table.access_matrix_element(matrix_access)?;
        match symbol_type {
            IdentifierType::Constant(ConstantType::Matrix(_)) => {
                let trace_segment = 0;
                let node_index = self.insert_op(Operation::Constant(ConstantValue::Matrix(
                    matrix_access.clone(),
                )));
                Ok((trace_segment, node_index))
            }
            IdentifierType::TransitionVariable(transition_variable) => {
                if let TransitionVariableType::Matrix(matrix) = transition_variable.value() {
                    let expr = &matrix[matrix_access.row_idx()][matrix_access.col_idx()];
                    if let Some((trace_segment, node_index)) =
                        variable_roots.get(&VariableValue::Matrix(matrix_access.clone()))
                    {
                        Ok((*trace_segment, *node_index))
                    } else {
                        let (trace_segment, node_index) =
                            self.insert_expr(symbol_table, expr.clone(), variable_roots)?;
                        variable_roots.insert(
                            VariableValue::Matrix(matrix_access.clone()),
                            (trace_segment, node_index),
                        );
                        Ok((trace_segment, node_index))
                    }
                } else {
                    Err(SemanticError::invalid_matrix_access(
                        matrix_access,
                        symbol_type,
                    ))
                }
            }
            _ => Err(SemanticError::invalid_matrix_access(
                matrix_access,
                symbol_type,
            )),
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

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum VariableValue {
    Scalar(String),
    Vector(VectorAccess),
    Matrix(MatrixAccess),
}
