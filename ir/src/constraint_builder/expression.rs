use super::{
    get_variable_expr, AccessType, BindingAccess, ConstraintBuilder, Expression, ListFolding,
    NodeIndex, Operation, SemanticError, SymbolType, TraceAccess, TraceBindingAccess, Value,
};

impl ConstraintBuilder {
    /// Combines two subgraphs representing equal subexpressions and returns the [ExprDetails] of
    /// the new subgraph.
    ///
    /// TODO: we can optimize this in the future in the case where lhs or rhs equals zero to just
    /// return the other expression.
    pub(crate) fn merge_equal_exprs(&mut self, lhs: NodeIndex, rhs: NodeIndex) -> NodeIndex {
        self.insert_graph_node(Operation::Sub(lhs, rhs))
    }

    /// Adds the expression to the graph and returns the [ExprDetails] of the constraint.
    /// Expressions are added recursively to reuse existing matching nodes.
    pub(crate) fn insert_expr(&mut self, expr: Expression) -> Result<NodeIndex, SemanticError> {
        match expr {
            // --- INLINE VALUES ------------------------------------------------------------------
            Expression::Const(value) => self.insert_inline_constant(value),

            // --- TRACE ACCESS REFERENCE ---------------------------------------------------------
            Expression::TraceAccess(column_access) => self.insert_trace_access(&column_access),
            Expression::TraceBindingAccess(trace_access) => {
                self.insert_trace_binding_access(trace_access)
            }

            // --- IDENTIFIER EXPRESSIONS ---------------------------------------------------------
            Expression::BindingAccess(access) => self.insert_symbol_access(access),
            Expression::Rand(ident, index) => {
                // TODO: replace Rand with BindingAccess in parser?
                let access_type = AccessType::Vector(index);
                let access = BindingAccess::new(ident, access_type);
                self.insert_symbol_access(access)
            }
            Expression::ListFolding(lf_type) => self.insert_list_folding(lf_type),

            // --- OPERATION EXPRESSIONS ----------------------------------------------------------
            Expression::Add(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(*lhs)?;
                let rhs = self.insert_expr(*rhs)?;
                // add the expression.
                let node_index = self.insert_graph_node(Operation::Add(lhs, rhs));
                Ok(node_index)
            }
            Expression::Sub(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(*lhs)?;
                let rhs = self.insert_expr(*rhs)?;
                // add the expression.
                let node_index = self.insert_graph_node(Operation::Sub(lhs, rhs));
                Ok(node_index)
            }
            Expression::Mul(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(*lhs)?;
                let rhs = self.insert_expr(*rhs)?;
                // add the expression.
                let node_index = self.insert_graph_node(Operation::Mul(lhs, rhs));
                Ok(node_index)
            }
            Expression::Exp(lhs, rhs) => self.insert_exp_op(*lhs, *rhs),
        }
    }

    // --- INLINE VALUES --------------------------------------------------------------------------

    /// Inserts the specified constant value into the graph and returns the resulting expression
    /// details.
    fn insert_inline_constant(&mut self, value: u64) -> Result<NodeIndex, SemanticError> {
        let node_index = self.insert_graph_node(Operation::Value(Value::InlineConstant(value)));

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
        trace_access: &TraceAccess,
    ) -> Result<NodeIndex, SemanticError> {
        self.symbol_table.validate_trace_access(trace_access)?;

        let node_index =
            self.insert_graph_node(Operation::Value(Value::TraceElement(*trace_access)));
        Ok(node_index)
    }

    pub(crate) fn insert_trace_binding_access(
        &mut self,
        trace_binding_access: TraceBindingAccess,
    ) -> Result<NodeIndex, SemanticError> {
        let trace_access = self
            .symbol_table
            .get_trace_binding_access(&trace_binding_access)?;
        self.insert_trace_access(&trace_access)
    }

    // --- OPERATOR EXPRESSIONS -----------------------------------------------------------------

    // TODO: docs
    fn insert_exp_op(
        &mut self,
        lhs: Expression,
        rhs: Expression,
    ) -> Result<NodeIndex, SemanticError> {
        // add base subexpression.
        let lhs = self.insert_expr(lhs)?;
        // add exponent subexpression.
        let node_index = if let Expression::Const(rhs) = rhs {
            self.insert_graph_node(Operation::Exp(lhs, rhs as usize))
        } else {
            Err(SemanticError::InvalidUsage(
                "Non const exponents are only allowed inside list comprehensions".to_string(),
            ))?
        };

        Ok(node_index)
    }

    // --- IDENTIFIER EXPRESSIONS -----------------------------------------------------------------

    /// Adds a trace column, periodic column, random value, named constant or a variable to the
    /// graph and returns the [NodeIndex] of the inserted expression.
    ///
    /// # Errors
    /// Returns an error if the identifier is not present in the symbol table or is not a supported
    /// type.
    fn insert_symbol_access(
        &mut self,
        binding_access: BindingAccess,
    ) -> Result<NodeIndex, SemanticError> {
        let symbol = self.symbol_table.get_symbol(binding_access.name())?;

        match symbol.symbol_type() {
            SymbolType::VariableBinding(bound_value) => {
                // access the expression bound to the variable and return an expression that reduces
                // to a single element.
                let expr = get_variable_expr(bound_value, binding_access)?;
                self.insert_expr(expr)
            }
            _ => {
                let (_, access_type) = binding_access.into_parts();
                // all other symbol types indicate we're accessing a value or group of values.
                let value = symbol.get_value(access_type)?;

                // add a value node in the graph.
                let node_index = self.insert_graph_node(Operation::Value(value));

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
    fn insert_list_folding(&mut self, lf_type: ListFolding) -> Result<NodeIndex, SemanticError> {
        match &lf_type {
            ListFolding::Sum(lf_value_type) | ListFolding::Prod(lf_value_type) => {
                let list = self.build_list_from_list_folding_value(lf_value_type)?;
                if list.is_empty() {
                    return Err(SemanticError::list_folding_empty_list(lf_value_type));
                }

                let mut acc = self.insert_expr(list[0].clone())?;
                for elem in list.into_iter().skip(1) {
                    let expr = self.insert_expr(elem)?;
                    let op = match lf_type {
                        ListFolding::Sum(_) => Operation::Add(acc, expr),
                        ListFolding::Prod(_) => Operation::Mul(acc, expr),
                    };
                    acc = self.insert_graph_node(op);
                }

                Ok(acc)
            }
        }
    }
}
