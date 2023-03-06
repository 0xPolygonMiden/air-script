use super::{
    build_list_from_list_folding_value, get_variable_expr, AccessType, BTreeMap, ConstantValue,
    ConstraintDomain, Expression, IndexedTraceAccess, IntegrityConstraintDegree, ListFoldingType,
    SemanticError, SymbolTable, SymbolType, TraceSegment, Value, AUX_SEGMENT, DEFAULT_SEGMENT,
};

// ALGEBRAIC GRAPH
// ================================================================================================

/// The AlgebraicGraph is a directed acyclic graph used to represent integrity constraints. To
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
    pub fn degree(&self, index: &NodeIndex) -> IntegrityConstraintDegree {
        let mut cycles: BTreeMap<usize, usize> = BTreeMap::new();
        let base = self.accumulate_degree(&mut cycles, index);

        if cycles.is_empty() {
            IntegrityConstraintDegree::new(base)
        } else {
            IntegrityConstraintDegree::with_cycles(base, cycles.values().cloned().collect())
        }
    }

    /// TODO: docs
    pub fn node_details(
        &self,
        index: &NodeIndex,
        default_domain: ConstraintDomain,
    ) -> Result<(TraceSegment, ConstraintDomain), SemanticError> {
        // recursively walk the subgraph and infer the trace segment and domain
        match self.node(index).op() {
            Operation::Value(value) => match value {
                Value::Constant(_) => Ok((DEFAULT_SEGMENT, default_domain)),
                Value::PeriodicColumn(_, _) => {
                    if default_domain.is_boundary() {
                        return Err(SemanticError::invalid_periodic_column_access_in_bc());
                    }
                    // the default domain for [IntegrityConstraints] is `EveryRow`
                    Ok((DEFAULT_SEGMENT, ConstraintDomain::EveryRow))
                }
                Value::PublicInput(_, _) => {
                    if default_domain.is_integrity() {
                        return Err(SemanticError::invalid_public_input_access_in_ic());
                    }
                    Ok((DEFAULT_SEGMENT, default_domain))
                }
                Value::RandomValue(_) => Ok((AUX_SEGMENT, default_domain)),
                Value::TraceElement(trace_access) => {
                    let domain = if default_domain.is_boundary() {
                        if trace_access.row_offset() == 0 {
                            default_domain
                        } else {
                            return Err(SemanticError::invalid_trace_offset_in_bc(trace_access));
                        }
                    } else {
                        trace_access.row_offset().into()
                    };

                    Ok((trace_access.trace_segment(), domain))
                }
            },
            Operation::Add(lhs, rhs) | Operation::Sub(lhs, rhs) | Operation::Mul(lhs, rhs) => {
                let (lhs_segment, lhs_domain) = self.node_details(lhs, default_domain)?;
                let (rhs_segment, rhs_domain) = self.node_details(rhs, default_domain)?;

                let trace_segment = lhs_segment.max(rhs_segment);
                let domain = lhs_domain.merge(&rhs_domain)?;

                Ok((trace_segment, domain))
            }
            Operation::Exp(lhs, _) => self.node_details(lhs, default_domain),
        }
    }

    // --- PUBLIC MUTATORS ------------------------------------------------------------------------

    /// Combines two subgraphs representing equal subexpressions and returns the [ExprDetails] of
    /// the new subgraph.
    ///
    /// TODO: we can optimize this in the future in the case where lhs or rhs equals zero to just
    /// return the other expression.
    pub(crate) fn merge_equal_exprs(&mut self, lhs: NodeIndex, rhs: NodeIndex) -> NodeIndex {
        self.insert_op(Operation::Sub(lhs, rhs))
    }

    /// Adds the expression to the graph and returns the [ExprDetails] of the constraint.
    /// Expressions are added recursively to reuse existing matching nodes.
    pub(crate) fn insert_expr(
        &mut self,
        symbol_table: &SymbolTable,
        expr: &Expression,
    ) -> Result<NodeIndex, SemanticError> {
        match expr {
            // --- INLINE VALUES ------------------------------------------------------------------
            Expression::Const(value) => self.insert_inline_constant(*value),

            // --- TRACE ACCESS REFERENCE ---------------------------------------------------------
            Expression::IndexedTraceAccess(column_access) => {
                self.insert_trace_access(symbol_table, column_access)
            }
            Expression::NamedTraceAccess(trace_access) => {
                let trace_access = symbol_table.get_trace_access_by_name(trace_access)?;
                self.insert_trace_access(symbol_table, &trace_access)
            }

            // --- IDENTIFIER EXPRESSIONS ---------------------------------------------------------
            Expression::Elem(ident) => {
                self.insert_symbol_access(symbol_table, ident.name(), AccessType::Default)
            }
            Expression::Rand(ident, index) => {
                let access_type = AccessType::Vector(*index);
                self.insert_symbol_access(symbol_table, ident.name(), access_type)
            }
            Expression::VectorAccess(vector_access) => {
                let access_type = AccessType::Vector(vector_access.idx());
                self.insert_symbol_access(symbol_table, vector_access.name(), access_type)
            }
            Expression::MatrixAccess(matrix_access) => {
                let access_type =
                    AccessType::Matrix(matrix_access.row_idx(), matrix_access.col_idx());
                self.insert_symbol_access(symbol_table, matrix_access.name(), access_type)
            }
            Expression::ListFolding(lf_type) => self.insert_list_folding(symbol_table, lf_type),

            // --- OPERATION EXPRESSIONS ----------------------------------------------------------
            Expression::Add(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(symbol_table, lhs)?;
                let rhs = self.insert_expr(symbol_table, rhs)?;
                // add the expression.
                let node_index = self.insert_op(Operation::Add(lhs, rhs));
                Ok(node_index)
            }
            Expression::Sub(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(symbol_table, lhs)?;
                let rhs = self.insert_expr(symbol_table, rhs)?;
                // add the expression.
                let node_index = self.insert_op(Operation::Sub(lhs, rhs));
                Ok(node_index)
            }
            Expression::Mul(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(symbol_table, lhs)?;
                let rhs = self.insert_expr(symbol_table, rhs)?;
                // add the expression.
                let node_index = self.insert_op(Operation::Mul(lhs, rhs));
                Ok(node_index)
            }
            Expression::Exp(lhs, rhs) => self.insert_exp_op(symbol_table, lhs, rhs),
        }
    }

    // --- INLINE VALUES --------------------------------------------------------------------------

    /// Inserts the specified constant value into the graph and returns the resulting expression
    /// details.
    fn insert_inline_constant(&mut self, value: u64) -> Result<NodeIndex, SemanticError> {
        let node_index = self.insert_op(Operation::Value(Value::Constant(ConstantValue::Inline(
            value,
        ))));

        Ok(node_index)
    }

    // --- TRACE ACCESS REFERENCE -----------------------------------------------------------------

    /// Adds a trace element access to the graph and returns the node index, trace segment, and row
    /// offset.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The column index of the trace access is greater than overall number of columns in segment.
    /// - The segment of the trace access is greater than the number of segments.
    pub(crate) fn insert_trace_access(
        &mut self,
        symbol_table: &SymbolTable,
        trace_access: &IndexedTraceAccess,
    ) -> Result<NodeIndex, SemanticError> {
        symbol_table.validate_trace_access(trace_access)?;

        let node_index = self.insert_op(Operation::Value(Value::TraceElement(*trace_access)));
        Ok(node_index)
    }

    // --- OPERATOR EXPRESSIONS -----------------------------------------------------------------

    // TODO: docs
    fn insert_exp_op(
        &mut self,
        symbol_table: &SymbolTable,
        lhs: &Expression,
        rhs: &Expression,
    ) -> Result<NodeIndex, SemanticError> {
        // add base subexpression.
        let lhs = self.insert_expr(symbol_table, lhs)?;
        // add exponent subexpression.
        let node_index = if let Expression::Const(rhs) = *rhs {
            self.insert_op(Operation::Exp(lhs, rhs as usize))
        } else {
            Err(SemanticError::InvalidUsage(
                "Non const exponents are only allowed inside list comprehensions".to_string(),
            ))?
        };

        Ok(node_index)
    }

    // --- IDENTIFIER EXPRESSIONS -----------------------------------------------------------------

    /// Adds a trace column, periodic column, random value, named constant or a variable to the
    /// graph and returns the [ExprDetails] of the inserted expression.
    ///
    /// # Errors
    /// Returns an error if the identifier is not present in the symbol table or is not a supported
    /// type.
    fn insert_symbol_access(
        &mut self,
        symbol_table: &SymbolTable,
        name: &str,
        access_type: AccessType,
    ) -> Result<NodeIndex, SemanticError> {
        let symbol = symbol_table.get_symbol(name)?;

        match symbol.symbol_type() {
            SymbolType::Variable(variable_type) => {
                // this symbol refers to an expression or group of expressions
                let expr = get_variable_expr(symbol.name(), variable_type, &access_type)?;
                self.insert_expr(symbol_table, &expr)
            }
            _ => {
                // all other symbol types indicate we're accessing a value or group of values.
                let value = symbol.get_value(&access_type)?;

                // add a value node in the graph.
                let node_index = self.insert_op(Operation::Value(value));

                Ok(node_index)
            }
        }
    }

    /// Inserts a list folding expression into the graph and returns the resulting expression
    /// details.
    ///
    /// # Errors
    /// - Panics if the list is empty.
    /// - Returns an error if the list cannot be unfolded properly.
    fn insert_list_folding(
        &mut self,
        symbol_table: &SymbolTable,
        lf_type: &ListFoldingType,
    ) -> Result<NodeIndex, SemanticError> {
        match lf_type {
            ListFoldingType::Sum(lf_value_type) | ListFoldingType::Prod(lf_value_type) => {
                let list = build_list_from_list_folding_value(lf_value_type, symbol_table)?;
                if list.is_empty() {
                    return Err(SemanticError::list_folding_empty_list(lf_value_type));
                }

                let mut acc = self.insert_expr(symbol_table, &list[0])?;
                for elem in list.iter().skip(1) {
                    let expr = self.insert_expr(symbol_table, elem)?;
                    let op = match lf_type {
                        ListFoldingType::Sum(_) => Operation::Add(acc, expr),
                        ListFoldingType::Prod(_) => Operation::Mul(acc, expr),
                    };
                    acc = self.insert_op(op);
                }

                Ok(acc)
            }
        }
    }

    // --- HELPERS --------------------------------------------------------------------------------

    /// Recursively accumulates the base degree and the cycle lengths of the periodic columns.
    fn accumulate_degree(&self, cycles: &mut BTreeMap<usize, usize>, index: &NodeIndex) -> usize {
        // recursively walk the subgraph and compute the degree from the operation and child nodes
        match self.node(index).op() {
            Operation::Value(value) => match value {
                Value::Constant(_) | Value::RandomValue(_) | Value::PublicInput(_, _) => 0,
                Value::TraceElement(_) => 1,
                Value::PeriodicColumn(index, cycle_len) => {
                    cycles.insert(*index, *cycle_len);
                    0
                }
            },
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

/// An integrity constraint operation or value reference.
#[derive(Debug, Eq, PartialEq)]
pub enum Operation {
    /// TODO: docs
    Value(Value),
    /// Addition operation applied to the nodes with the specified indices.
    Add(NodeIndex, NodeIndex),
    /// Subtraction operation applied to the nodes with the specified indices.
    Sub(NodeIndex, NodeIndex),
    /// Multiplication operation applied to the nodes with the specified indices.
    Mul(NodeIndex, NodeIndex),
    /// Exponentiation operation applied to the node with the specified index, using the provided
    /// value as the power.
    /// TODO: Support non const exponents.
    Exp(NodeIndex, usize),
}

impl Operation {
    pub fn precedence(&self) -> usize {
        match self {
            Operation::Add(_, _) => 1,
            Operation::Sub(_, _) => 2,
            Operation::Mul(_, _) => 3,
            _ => 4,
        }
    }
}
