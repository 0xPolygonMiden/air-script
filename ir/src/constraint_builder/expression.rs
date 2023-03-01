use super::{
    AccessType, ConstraintBuilder, ConstraintDomain, ExprDetails,ConstantValue,
    Expression, Identifier, IndexedTraceAccess, ListFoldingType, MatrixAccess, Operation,
    SemanticError, SymbolAccess, SymbolType, VariableType, VectorAccess, DEFAULT_SEGMENT, Value
};

impl ConstraintBuilder {
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
        let node_index = self
            .constraints
            .insert_graph_node(Operation::Sub(lhs.root_idx(), rhs.root_idx()));
        let trace_segment = lhs.trace_segment().max(rhs.trace_segment());
        let domain = lhs.domain().merge(&rhs.domain())?;

        Ok(ExprDetails::new(node_index, trace_segment, domain))
    }

    /// Adds the expression to the graph and returns the [ExprDetails] of the constraint.
    /// Expressions are added recursively to reuse existing matching nodes.
    /// TODO: update docs
    pub(super) fn insert_expr(
        &mut self,
        expr: &Expression,
        default_domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        match expr {
            // --- INLINE VALUES ------------------------------------------------------------------
            Expression::Const(value) => {
                // TODO: restore insert_constant function
                let node_index = self
                    .constraints
                    .insert_graph_node(Operation::Value(Value::Constant(ConstantValue::Inline(*value))));
                Ok(ExprDetails::new(
                    node_index,
                    DEFAULT_SEGMENT,
                    default_domain,
                ))
            }
            Expression::IndexedTraceAccess(trace_access) => {
                self.insert_trace_access(trace_access, default_domain)
            }

            // --- IDENTIFIER EXPRESSIONS ---------------------------------------------------------
            Expression::Elem(ident) => {
                self.insert_symbol_access(ident.name(), AccessType::Default, default_domain)
            },
            Expression::NamedTraceAccess(trace_access) => {
                let access_type = AccessType::Vector(trace_access.idx());
                self.insert_symbol_access(trace_access.name(), access_type, default_domain)
            }
            Expression::Rand(ident, index) => {
                let access_type = AccessType::Vector(*index);
                self.insert_symbol_access(ident.name(), access_type, default_domain)
            }
            Expression::VectorAccess(vector_access) => {
                let access_type = AccessType::Vector(vector_access.idx());
                self.insert_symbol_access(vector_access.name(), access_type, default_domain)
            }
            Expression::MatrixAccess(matrix_access) => {
                let access_type = AccessType::Matrix(matrix_access.row_idx(), matrix_access.col_idx());
                self.insert_symbol_access(matrix_access.name(), access_type, default_domain)
            }
            Expression::ListFolding(lf_type) => self.insert_list_folding(lf_type, default_domain),

            // --- OPERATION EXPRESSIONS ----------------------------------------------------------
            Expression::Add(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(lhs, default_domain)?;
                let rhs = self.insert_expr(rhs, default_domain)?;
                // add the expression.
                self.insert_bin_op(&lhs, &rhs, Operation::Add(lhs.root_idx(), rhs.root_idx()))
            }
            Expression::Sub(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(lhs, default_domain)?;
                let rhs = self.insert_expr(rhs, default_domain)?;
                // add the expression.
                self.insert_bin_op(&lhs, &rhs, Operation::Sub(lhs.root_idx(), rhs.root_idx()))
            }
            Expression::Mul(lhs, rhs) => {
                // add both subexpressions.
                let lhs = self.insert_expr(lhs, default_domain)?;
                let rhs = self.insert_expr(rhs, default_domain)?;
                // add the expression.
                self.insert_bin_op(&lhs, &rhs, Operation::Mul(lhs.root_idx(), rhs.root_idx()))
            }
            Expression::Exp(lhs, rhs) => {
                // add base subexpression.
                let lhs = self.insert_expr(lhs, default_domain)?;
                // add exponent subexpression.
                let node_index = if let Expression::Const(rhs) = **rhs {
                    self.constraints
                        .insert_graph_node(Operation::Exp(lhs.root_idx(), rhs as usize))
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
        }
    }

    /// Inserts a binary operation into the graph and returns the resulting expression details.
    pub(super) fn insert_bin_op(
        &mut self,
        lhs: &ExprDetails,
        rhs: &ExprDetails,
        op: Operation,
    ) -> Result<ExprDetails, SemanticError> {
        let node_index = self.constraints.insert_graph_node(op);
        let trace_segment = lhs.trace_segment().max(rhs.trace_segment());
        let domain = lhs.domain().merge(&rhs.domain())?;

        Ok(ExprDetails::new(node_index, trace_segment, domain))
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
        trace_access: &IndexedTraceAccess,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        self.symbol_table.validate_trace_access(trace_access)?;

        let trace_segment = trace_access.trace_segment();
        let node_index = self
            .constraints
            .insert_graph_node(Operation::Value(Value::TraceElement(*trace_access)));
        Ok(ExprDetails::new(node_index, trace_segment, domain))
    }

    // --- IDENTIFIER EXPRESSIONS -----------------------------------------------------------------

    /// Adds a trace column, periodic column, random value, named constant or a variable to the
    /// graph and returns the [ExprDetails] of the inserted expression.
    ///
    /// # Errors
    /// Returns an error if the identifier is not present in the symbol table or is not a supported
    /// type.
    pub(super) fn insert_symbol_access(
        &mut self,
        name: &str,
        access_type: AccessType,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        let current_scope = domain.into();
        let symbol = self.symbol_table.get_symbol(name, current_scope)?;

        match symbol.symbol_type() {
            SymbolType::Variable(variable_type) => {
                symbol.validate_access(&access_type)?;
                // this symbol refers to an expression or group of expressions
                let symbol_access = SymbolAccess::new(symbol.clone(), access_type);
                // self.insert_variable(symbol_access, variable_type, domain)
                
                if let Some(expr_details) = self.variable_roots.get(&symbol_access) {
                    // TODO: deal with boundary conflict properly
                    if expr_details.domain().is_boundary() {
                        Ok(ExprDetails::new(expr_details.root_idx(), expr_details.trace_segment(), domain))
                    } else {
                        Ok(*expr_details)
                    }
                } else {
                    // Otherwise, insert the variable expression and create a new variable root.
                    let expr = match (variable_type, symbol_access.access_type()) {
                        (VariableType::Scalar(expr), AccessType::Default) => expr.clone(),
                        (VariableType::Scalar(expr), AccessType::Vector(idx)) => match expr {
                            Expression::Elem(elem) => {
                                Expression::VectorAccess(VectorAccess::new(elem.clone(), *idx))
                            }
                            Expression::VectorAccess(matrix_row_access) => {
                                Expression::MatrixAccess(MatrixAccess::new(
                                    Identifier(matrix_row_access.name().to_string()),
                                    matrix_row_access.idx(),
                                    *idx,
                                ))
                            }
                            _ => {
                                // TODO: replace this error
                                // return  Err(SemanticError::invalid_vector_access(
                                //     vector_access,
                                //     symbol_access.symbol().symbol_type(),
                                // ))
                                return Err(SemanticError::InvalidUsage(format!(
                                    "Invalid variable access for variable type {variable_type:?} and symbol access {symbol_access:?}",
                                )));
                            }
                        },
                        (VariableType::Scalar(Expression::Elem(elem)), AccessType::Matrix(row_idx, col_idx)) => {
                            Expression::MatrixAccess(MatrixAccess::new(
                                elem.clone(),
                                *row_idx,
                                *col_idx,
                            ))
                        }
                        (VariableType::Vector(expr_vector), AccessType::Vector(idx)) => expr_vector[*idx].clone(),
                        (VariableType::Vector(expr_vector), AccessType::Matrix(row_idx, col_idx)) => match &expr_vector[*row_idx] {
                            Expression::Elem(elem) => {
                                Expression::VectorAccess(VectorAccess::new(elem.clone(), *col_idx))
                            }
                            Expression::VectorAccess(matrix_row_access) => {
                                Expression::MatrixAccess(MatrixAccess::new(
                                    Identifier(matrix_row_access.name().to_string()),
                                    matrix_row_access.idx(),
                                    *col_idx,
                                ))
                            }
                            _ => 
                                // TODO: replace this error
                                // Err(SemanticError::invalid_matrix_access(
                                //     matrix_access,
                                //     symbol_access.symbol().symbol_type(),
                                // )),
                                return Err(SemanticError::InvalidUsage(format!(
                                "Invalid variable access for variable type {variable_type:?} and symbol access {symbol_access:?}",
                            )))
                        }
                        (VariableType::Matrix(expr_matrix), AccessType::Matrix(row_idx, col_idx)) => {
                            expr_matrix[*row_idx][*col_idx].clone()
                        }
                        _ => {
                            // TODO: update this error
                            return Err(SemanticError::InvalidUsage(format!(
                                "Invalid variable access for variable type {variable_type:?} and symbol access {symbol_access:?}",
                            )));
                        }
                    };

                    let expr_details = self.insert_expr( &expr, domain)?;
                    self.variable_roots.insert(symbol_access, expr_details);

                    Ok(expr_details)
                }
            }
            _ => {
                // all other symbol types indicate we're accessing a value or group of values.
                let value = symbol.access_value(access_type)?;
                let trace_segment = symbol.trace_segment();

                // add a value node in the graph.
                let node_index = self
                .constraints
                .insert_graph_node(Operation::Value(value));
        
                // TODO: fix ExprDetails segment and domain
                Ok(ExprDetails::new(node_index, trace_segment, domain))
            }
        }
    }
    
    /// Looks up the specified variable value in the variable roots and returns the expression
    /// details if it is found. Otherwise, inserts the variable expression into the graph, adds it
    /// to the variable roots, and returns the resulting expression details.
    pub(super) fn insert_variable(
        &mut self,
        symbol_access: SymbolAccess,
        variable_type: &VariableType,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        if let Some(expr_details) = self.variable_roots.get(&symbol_access) {
            // TODO: deal with boundary conflict properly
            if expr_details.domain().is_boundary() {
                Ok(ExprDetails::new(expr_details.root_idx(), expr_details.trace_segment(), domain))
            } else {
                Ok(*expr_details)
            }
        } else {
            // Otherwise, insert the variable expression and create a new variable root.
            let expr = match (variable_type, symbol_access.access_type()) {
                (VariableType::Scalar(expr), AccessType::Default) => expr.clone(),
                (VariableType::Scalar(expr), AccessType::Vector(idx)) => match expr {
                    Expression::Elem(elem) => {
                        Expression::VectorAccess(VectorAccess::new(elem.clone(), *idx))
                    }
                    Expression::VectorAccess(matrix_row_access) => {
                        Expression::MatrixAccess(MatrixAccess::new(
                            Identifier(matrix_row_access.name().to_string()),
                            matrix_row_access.idx(),
                            *idx,
                        ))
                    }
                    _ => {
                        // TODO: replace this error
                        // return  Err(SemanticError::invalid_vector_access(
                        //     vector_access,
                        //     symbol_access.symbol().symbol_type(),
                        // ))
                        return Err(SemanticError::InvalidUsage(format!(
                            "Invalid variable access for variable type {variable_type:?} and symbol access {symbol_access:?}",
                        )));
                    }
                },
                (VariableType::Scalar(Expression::Elem(elem)), AccessType::Matrix(row_idx, col_idx)) => {
                    Expression::MatrixAccess(MatrixAccess::new(
                        elem.clone(),
                        *row_idx,
                        *col_idx,
                    ))
                }
                (VariableType::Vector(expr_vector), AccessType::Vector(idx)) => expr_vector[*idx].clone(),
                (VariableType::Vector(expr_vector), AccessType::Matrix(row_idx, col_idx)) => match &expr_vector[*row_idx] {
                    Expression::Elem(elem) => {
                        Expression::VectorAccess(VectorAccess::new(elem.clone(), *col_idx))
                    }
                    Expression::VectorAccess(matrix_row_access) => {
                        Expression::MatrixAccess(MatrixAccess::new(
                            Identifier(matrix_row_access.name().to_string()),
                            matrix_row_access.idx(),
                            *col_idx,
                        ))
                    }
                    _ => 
                        // TODO: replace this error
                        // Err(SemanticError::invalid_matrix_access(
                        //     matrix_access,
                        //     symbol_access.symbol().symbol_type(),
                        // )),
                        return Err(SemanticError::InvalidUsage(format!(
                        "Invalid variable access for variable type {variable_type:?} and symbol access {symbol_access:?}",
                    )))
                }
                (VariableType::Matrix(expr_matrix), AccessType::Matrix(row_idx, col_idx)) => {
                    expr_matrix[*row_idx][*col_idx].clone()
                }
                _ => {
                    // TODO: update this error
                    return Err(SemanticError::InvalidUsage(format!(
                        "Invalid variable access for variable type {variable_type:?} and symbol access {symbol_access:?}",
                    )));
                }
            };

            let expr_details = self.insert_expr( &expr, domain)?;
            self.variable_roots.insert(symbol_access, expr_details);

            Ok(expr_details)
        }
    }

    /// Inserts a list folding expression into the graph and returns the resulting expression
    /// details.
    ///
    /// # Errors
    /// - Panics if the list is empty.
    /// - Returns an error if the list cannot be unfolded properly.
    pub(super) fn insert_list_folding(
        &mut self,
        lf_type: &ListFoldingType,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        match lf_type {
            ListFoldingType::Sum(lf_value_type) | ListFoldingType::Prod(lf_value_type) => {
                let list = self
                    .symbol_table
                    .build_list_from_list_folding_value(lf_value_type)?;
                if list.is_empty() {
                    return Err(SemanticError::list_folding_empty_list(lf_value_type));
                }

                let mut acc = self.insert_expr(&list[0], domain)?;
                for elem in list.iter().skip(1) {
                    let expr = self.insert_expr(elem, domain)?;
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