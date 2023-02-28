use super::{
    list_comprehension::unfold_lc, BTreeMap, ConstantType, ConstraintDomain, ExprDetails,
    Expression, Identifier, IdentifierType, IndexedTraceAccess, IntegrityConstraintDegree,
    ListFoldingType, MatrixAccess, Scope, SemanticError, SymbolTable, TraceSegment, VariableRoots,
    VariableType, VectorAccess,
};

// CONSTANTS
// ================================================================================================

/// The offset of the "current" row during constraint evaluation.
pub const CURRENT_ROW: usize = 0;
/// The default segment against which a constraint is applied is the main trace segment.
pub const DEFAULT_SEGMENT: TraceSegment = 0;
/// The auxiliary trace segment.
const AUX_SEGMENT: TraceSegment = 1;

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

    // --- PUBLIC MUTATORS ------------------------------------------------------------------------

    /// Combines two subgraphs representing equal subexpressions and returns the [ExprDetails] of
    /// the new subgraph.
    ///
    /// TODO: we can optimize this in the future in the case where lhs or rhs equals zero to just
    /// return the other expression.
    pub(super) fn merge_equal_exprs(
        &mut self,
        lhs: &ExprDetails,
        rhs: &ExprDetails,
    ) -> Result<ExprDetails, SemanticError> {
        let node_index = self.insert_op(Operation::Sub(lhs.root_idx(), rhs.root_idx()));
        let trace_segment = lhs.trace_segment().max(rhs.trace_segment());
        let domain = lhs.domain().merge(&rhs.domain())?;

        Ok(ExprDetails::new(node_index, trace_segment, domain))
    }

    /// Adds the expression to the graph and returns the [ExprDetails] of the constraint.
    /// Expressions are added recursively to reuse existing matching nodes.
    pub(super) fn insert_expr(
        &mut self,
        symbol_table: &SymbolTable,
        expr: &Expression,
        variable_roots: &mut VariableRoots,
        default_domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        match expr {
            Expression::Const(value) => {
                let node_index = self.insert_op(Operation::Constant(ConstantValue::Inline(*value)));
                Ok(ExprDetails::new(
                    node_index,
                    DEFAULT_SEGMENT,
                    default_domain,
                ))
            }
            Expression::Elem(Identifier(ident)) => {
                self.insert_symbol_access(symbol_table, ident, variable_roots, default_domain)
            }
            Expression::VectorAccess(vector_access) => self.insert_vector_access(
                symbol_table,
                vector_access,
                variable_roots,
                default_domain,
            ),
            Expression::MatrixAccess(matrix_access) => self.insert_matrix_access(
                symbol_table,
                matrix_access,
                variable_roots,
                default_domain,
            ),
            Expression::Rand(name, index) => {
                // The constraint target for random values defaults to the second (auxiliary) trace
                // segment.
                // TODO: make this more general, so random values from further trace segments can be
                // used. This requires having a way to describe different sets of randomness in
                // the AirScript syntax.
                self.insert_random_access(
                    symbol_table,
                    name.name(),
                    *index,
                    AUX_SEGMENT,
                    default_domain,
                )
            }
            Expression::IndexedTraceAccess(column_access) => {
                self.insert_trace_access(symbol_table, column_access, default_domain)
            }
            Expression::NamedTraceAccess(trace_access) => {
                let trace_access = symbol_table.get_trace_access_by_name(trace_access)?;
                self.insert_trace_access(symbol_table, &trace_access, default_domain)
            }
            Expression::Add(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(symbol_table, lhs, variable_roots, default_domain)?;
                let rhs = self.insert_expr(symbol_table, rhs, variable_roots, default_domain)?;
                // add the expression.
                self.insert_bin_op(&lhs, &rhs, Operation::Add(lhs.root_idx(), rhs.root_idx()))
            }
            Expression::Sub(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(symbol_table, lhs, variable_roots, default_domain)?;
                let rhs = self.insert_expr(symbol_table, rhs, variable_roots, default_domain)?;
                // add the expression.
                self.insert_bin_op(&lhs, &rhs, Operation::Sub(lhs.root_idx(), rhs.root_idx()))
            }
            Expression::Mul(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(symbol_table, lhs, variable_roots, default_domain)?;
                let rhs = self.insert_expr(symbol_table, rhs, variable_roots, default_domain)?;
                // add the expression.
                self.insert_bin_op(&lhs, &rhs, Operation::Mul(lhs.root_idx(), rhs.root_idx()))
            }
            Expression::Exp(lhs, rhs) => {
                // add base subexpression.
                let lhs = self.insert_expr(symbol_table, lhs, variable_roots, default_domain)?;
                // add exponent subexpression.
                let node_index = if let Expression::Const(rhs) = **rhs {
                    self.insert_op(Operation::Exp(lhs.root_idx(), rhs as usize))
                } else {
                    Err(SemanticError::InvalidUsage(
                        "Non const exponents are only allowed inside list comprehensions"
                            .to_string(),
                    ))?
                };

                Ok(ExprDetails::new(
                    node_index,
                    lhs.trace_segment(),
                    lhs.domain(),
                ))
            }
            Expression::ListFolding(lf_type) => {
                self.insert_list_folding(symbol_table, lf_type, variable_roots, default_domain)
            }
        }
    }

    /// Adds a trace element access to the graph and returns the node index, trace segment, and row
    /// offset.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The column index of the trace access is greater than overall number of columns in segment.
    /// - The segment of the trace access is greater than the number of segments.
    pub(super) fn insert_trace_access(
        &mut self,
        symbol_table: &SymbolTable,
        trace_access: &IndexedTraceAccess,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        symbol_table.validate_trace_access(trace_access)?;

        let trace_segment = trace_access.trace_segment();
        let node_index = self.insert_op(Operation::TraceElement(*trace_access));
        Ok(ExprDetails::new(node_index, trace_segment, domain))
    }

    // --- HELPERS --------------------------------------------------------------------------------

    /// Recursively accumulates the base degree and the cycle lengths of the periodic columns.
    fn accumulate_degree(&self, cycles: &mut BTreeMap<usize, usize>, index: &NodeIndex) -> usize {
        // recursively walk the subgraph and compute the degree from the operation and child nodes
        match self.node(index).op() {
            Operation::Constant(_) | Operation::RandomValue(_) | Operation::PublicInput(_, _) => 0,
            Operation::TraceElement(_) => 1,
            Operation::PeriodicColumn(index, cycle_len) => {
                cycles.insert(*index, *cycle_len);
                0
            }
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

    /// Inserts a binary operation into the graph and returns the resulting expression details.
    fn insert_bin_op(
        &mut self,
        lhs: &ExprDetails,
        rhs: &ExprDetails,
        op: Operation,
    ) -> Result<ExprDetails, SemanticError> {
        let node_index = self.insert_op(op);
        let trace_segment = lhs.trace_segment().max(rhs.trace_segment());
        let domain = lhs.domain().merge(&rhs.domain())?;
        Ok(ExprDetails::new(node_index, trace_segment, domain))
    }

    /// Inserts the specified constant value into the graph and returns the resulting expression
    /// details.
    fn insert_constant(
        &mut self,
        constant: ConstantValue,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        let node_index = self.insert_op(Operation::Constant(constant));
        Ok(ExprDetails::new(node_index, DEFAULT_SEGMENT, domain))
    }

    /// Inserts random value with specified index into the graph and returns the resulting
    /// expression details.
    fn insert_random_value(
        &mut self,
        symbol_table: &SymbolTable,
        index: usize,
        trace_segment: u8,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        symbol_table.validate_rand_access(index)?;

        let node_index = self.insert_op(Operation::RandomValue(index));
        Ok(ExprDetails::new(node_index, trace_segment, domain))
    }

    /// Looks up the specified variable value in the variable roots and returns the expression
    /// details if it is found. Otherwise, inserts the variable expression into the graph, adds it
    /// to the variable roots, and returns the resulting expression details.
    fn insert_variable(
        &mut self,
        symbol_table: &SymbolTable,
        variable_roots: &mut VariableRoots,
        domain: ConstraintDomain,
        scope: Scope,
        variable_value: VariableValue,
        variable_expr: &Expression,
    ) -> Result<ExprDetails, SemanticError> {
        // The scope of the variable must be valid for the constraint domain.
        match (scope, domain) {
            (
                Scope::BoundaryConstraints,
                ConstraintDomain::FirstRow | ConstraintDomain::LastRow,
            )
            | (
                Scope::IntegrityConstraints,
                ConstraintDomain::EveryRow | ConstraintDomain::EveryFrame(_),
            ) => {
                let key = (scope, variable_value);
                if let Some(expr) = variable_roots.get(&key) {
                    match scope {
                        Scope::BoundaryConstraints => {
                            // TODO: deal with boundary conflict properly
                            Ok(ExprDetails::new(
                                expr.root_idx(),
                                expr.trace_segment(),
                                domain,
                            ))
                        }
                        Scope::IntegrityConstraints => Ok(*expr),
                    }
                } else {
                    // Otherwise, insert the variable expression and create a new variable root.
                    let expr =
                        self.insert_expr(symbol_table, variable_expr, variable_roots, domain)?;
                    variable_roots.insert(key, expr);
                    Ok(expr)
                }
            }
            (_, _) => Err(SemanticError::OutOfScope(format!(
                "Variable {variable_value:?} is out of scope",
            ))),
        }
    }

    /// Adds a trace column, periodic column, random value, named constant or a variable to the
    /// graph and returns the [ExprDetails] of the inserted expression.
    ///
    /// # Errors
    /// Returns an error if the identifier is not present in the symbol table or is not a supported
    /// type.
    fn insert_symbol_access(
        &mut self,
        symbol_table: &SymbolTable,
        ident: &str,
        variable_roots: &mut VariableRoots,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        let elem_type = symbol_table.get_type(ident)?;
        match elem_type {
            IdentifierType::Constant(ConstantType::Scalar(_)) => {
                self.insert_constant(ConstantValue::Scalar(ident.to_string()), domain)
            }
            IdentifierType::RandomValuesBinding(offset, _) => {
                self.insert_random_value(symbol_table, *offset, AUX_SEGMENT, domain)
            }
            IdentifierType::Variable(scope, variable) => {
                if let VariableType::Scalar(variable_expr) = variable.value() {
                    self.insert_variable(
                        symbol_table,
                        variable_roots,
                        domain,
                        *scope,
                        VariableValue::Scalar(ident.to_string()),
                        variable_expr,
                    )
                } else {
                    Err(SemanticError::InvalidUsage(format!(
                        "Identifier {ident} was declared as a {elem_type} which is not a supported type."
                    )))
                }
            }
            IdentifierType::PeriodicColumn(index, cycle_len) => {
                let node_index = self.insert_op(Operation::PeriodicColumn(*index, *cycle_len));
                Ok(ExprDetails::new(node_index, DEFAULT_SEGMENT, domain))
            }
            IdentifierType::TraceColumns(columns) => {
                let trace_segment = columns.trace_segment();
                let trace_access =
                    IndexedTraceAccess::new(trace_segment, columns.offset(), CURRENT_ROW);
                let node_index = self.insert_op(Operation::TraceElement(trace_access));
                Ok(ExprDetails::new(node_index, trace_segment, domain))
            }
            _ => Err(SemanticError::InvalidUsage(format!(
                "Identifier {ident} was declared as a {elem_type} which is not a supported type."
            ))),
        }
    }

    /// Validates and adds a vector access to the graph and returns the [ExprDetails] of the
    /// inserted expression.
    ///
    /// # Errors
    /// Returns an error if the identifier's value is not of a supported type.
    fn insert_vector_access(
        &mut self,
        symbol_table: &SymbolTable,
        vector_access: &VectorAccess,
        variable_roots: &mut VariableRoots,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        let symbol_type = symbol_table.access_vector_element(vector_access)?;
        match symbol_type {
            IdentifierType::Constant(ConstantType::Vector(_)) => {
                self.insert_constant(ConstantValue::Vector(vector_access.clone()), domain)
            }
            IdentifierType::Variable(scope, variable) => match variable.value() {
                VariableType::Scalar(matrix_vector_access_expr) => {
                    match matrix_vector_access_expr {
                        Expression::Elem(elem) => {
                            let equal_vector_access = Expression::VectorAccess(VectorAccess::new(
                                elem.clone(),
                                vector_access.idx(),
                            ));
                            self.insert_variable(
                                symbol_table,
                                variable_roots,
                                domain,
                                *scope,
                                VariableValue::Vector(vector_access.clone()),
                                &equal_vector_access,
                            )
                        }
                        Expression::VectorAccess(matrix_row_access) => {
                            let matrix_access = Expression::MatrixAccess(MatrixAccess::new(
                                Identifier(matrix_row_access.name().to_string()),
                                matrix_row_access.idx(),
                                vector_access.idx(),
                            ));
                            self.insert_variable(
                                symbol_table,
                                variable_roots,
                                domain,
                                *scope,
                                VariableValue::Vector(vector_access.clone()),
                                &matrix_access,
                            )
                        }
                        _ => Err(SemanticError::invalid_vector_access(
                            vector_access,
                            symbol_type,
                        )),
                    }
                }
                VariableType::Vector(vector) => self.insert_variable(
                    symbol_table,
                    variable_roots,
                    domain,
                    *scope,
                    VariableValue::Vector(vector_access.clone()),
                    &vector[vector_access.idx()],
                ),
                _ => Err(SemanticError::invalid_vector_access(
                    vector_access,
                    symbol_type,
                )),
            },
            IdentifierType::TraceColumns(columns) => {
                let trace_segment = columns.trace_segment();
                let trace_access = IndexedTraceAccess::new(
                    trace_segment,
                    columns.offset() + vector_access.idx(),
                    CURRENT_ROW,
                );
                let node_index = self.insert_op(Operation::TraceElement(trace_access));
                Ok(ExprDetails::new(node_index, trace_segment, domain))
            }
            IdentifierType::PublicInput(_) => {
                if !domain.is_boundary() {
                    return Err(SemanticError::InvalidUsage(
                        "Public inputs cannot be accessed in integrity constraints.".to_string(),
                    ));
                }
                let node_index = self.insert_op(Operation::PublicInput(
                    vector_access.name().to_string(),
                    vector_access.idx(),
                ));
                Ok(ExprDetails::new(node_index, DEFAULT_SEGMENT, domain))
            }
            IdentifierType::RandomValuesBinding(offset, _) => self.insert_random_value(
                symbol_table,
                *offset + vector_access.idx(),
                AUX_SEGMENT,
                domain,
            ),
            _ => Err(SemanticError::invalid_vector_access(
                vector_access,
                symbol_type,
            )),
        }
    }

    /// Validates and adds a matrix access to the graph and returns the [ExprDetails] of the
    /// inserted expression.
    ///
    /// # Errors
    /// Returns an error if the identifier's value is not of a supported type.
    fn insert_matrix_access(
        &mut self,
        symbol_table: &SymbolTable,
        matrix_access: &MatrixAccess,
        variable_roots: &mut VariableRoots,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        let symbol_type = symbol_table.access_matrix_element(matrix_access)?;
        match symbol_type {
            IdentifierType::Constant(ConstantType::Matrix(_)) => {
                self.insert_constant(ConstantValue::Matrix(matrix_access.clone()), domain)
            }
            IdentifierType::Variable(scope, variable) => match variable.value() {
                VariableType::Scalar(scalar) => {
                    if let Expression::Elem(elem) = scalar {
                        let equal_matrix_access = Expression::MatrixAccess(MatrixAccess::new(
                            elem.clone(),
                            matrix_access.row_idx(),
                            matrix_access.col_idx(),
                        ));
                        self.insert_variable(
                            symbol_table,
                            variable_roots,
                            domain,
                            *scope,
                            VariableValue::Matrix(matrix_access.clone()),
                            &equal_matrix_access,
                        )
                    } else {
                        Err(SemanticError::invalid_matrix_access(
                            matrix_access,
                            symbol_type,
                        ))
                    }
                }
                VariableType::Vector(vector) => {
                    let vec_elem = &vector[matrix_access.row_idx()];
                    match vec_elem {
                        Expression::Elem(elem) => {
                            let vector_access = Expression::VectorAccess(VectorAccess::new(
                                elem.clone(),
                                matrix_access.col_idx(),
                            ));
                            self.insert_variable(
                                symbol_table,
                                variable_roots,
                                domain,
                                *scope,
                                VariableValue::Matrix(matrix_access.clone()),
                                &vector_access,
                            )
                        }
                        Expression::VectorAccess(matrix_row_access) => {
                            let internal_matrix_access =
                                Expression::MatrixAccess(MatrixAccess::new(
                                    Identifier(matrix_row_access.name().to_string()),
                                    matrix_row_access.idx(),
                                    matrix_access.col_idx(),
                                ));
                            self.insert_variable(
                                symbol_table,
                                variable_roots,
                                domain,
                                *scope,
                                VariableValue::Matrix(matrix_access.clone()),
                                &internal_matrix_access,
                            )
                        }
                        _ => Err(SemanticError::invalid_matrix_access(
                            matrix_access,
                            symbol_type,
                        )),
                    }
                }
                VariableType::Matrix(matrix) => self.insert_variable(
                    symbol_table,
                    variable_roots,
                    domain,
                    *scope,
                    VariableValue::Matrix(matrix_access.clone()),
                    &matrix[matrix_access.row_idx()][matrix_access.col_idx()],
                ),
                _ => Err(SemanticError::invalid_matrix_access(
                    matrix_access,
                    symbol_type,
                )),
            },
            _ => Err(SemanticError::invalid_matrix_access(
                matrix_access,
                symbol_type,
            )),
        }
    }

    /// Inserts random value by index access into the graph and returns the resulting
    /// expression details.
    ///
    /// # Errors
    /// Returns an error if the identifier is not present in the symbol table or is not a supported
    /// type.
    ///
    /// # Example
    /// This function inserts values like `$alphas[3]`, having `name` as `alphas` and `index` as
    /// `3`
    fn insert_random_access(
        &mut self,
        symbol_table: &SymbolTable,
        name: &str,
        index: usize,
        trace_segment: u8,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        let elem_type = symbol_table.get_type(name)?;
        match elem_type {
            IdentifierType::RandomValuesBinding(_, _) => {
                self.insert_random_value(symbol_table, index, trace_segment, domain)
            }
            _ => Err(SemanticError::InvalidUsage(format!(
                "Identifier {name} was declared as a {elem_type} not as a random values"
            ))),
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
        variable_roots: &mut VariableRoots,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        match lf_type {
            ListFoldingType::Sum(lc) | ListFoldingType::Prod(lc) => {
                let list = unfold_lc(lc, symbol_table)?;
                assert!(
                    !list.is_empty(),
                    "List on which list folding is applied is empty."
                );

                let mut acc = self.insert_expr(symbol_table, &list[0], variable_roots, domain)?;
                for elem in list.iter().skip(1) {
                    let expr = self.insert_expr(symbol_table, elem, variable_roots, domain)?;
                    let op = match lf_type {
                        ListFoldingType::Sum(_) => Operation::Add(acc.root_idx(), expr.root_idx()),
                        ListFoldingType::Prod(_) => Operation::Mul(acc.root_idx(), expr.root_idx()),
                    };
                    acc = self.insert_bin_op(&acc, &expr, op)?;
                }

                Ok(acc)
            }
        }
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
    /// An inlined or named constant with identifier and access indices.
    Constant(ConstantValue),
    /// An identifier for an element in the trace segment, column, and row offset specified by the
    /// [IndexedTraceAccess]
    TraceElement(IndexedTraceAccess),
    /// An identifier for a periodic value from a specified periodic column. The first inner value
    /// is the index of the periodic column within the declared periodic columns. The second inner
    /// value is the length of the column's periodic cycle. The periodic value made available from
    /// the specified column is based on the current row of the trace.
    PeriodicColumn(usize, usize),
    /// An identifier for a public input declared by the specified name and accessed at the
    /// specified index.
    PublicInput(String, usize),
    /// A random value provided by the verifier. The inner value is the index of this random value
    /// in the array of all random values.
    RandomValue(usize),
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
