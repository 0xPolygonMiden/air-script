use super::{
    AccessType, BTreeMap, ConstantValue, ConstraintBuilder, ConstraintDomain, ExprDetails,
    Expression, Identifier, IndexedTraceAccess, ListFoldingType, MatrixAccess, Operation,
    SemanticError, Symbol, SymbolAccess, SymbolType, Value, VariableType, VectorAccess,
    DEFAULT_SEGMENT,
};

impl ConstraintBuilder {
    pub fn get_variable_expr(
        &self,
        symbol: &Symbol,
        access_type: AccessType,
        variable_type: &VariableType,
    ) -> Result<Expression, SemanticError> {
        symbol.validate_access(&access_type)?;

        let expr = match (variable_type, access_type) {
            (VariableType::Scalar(expr), AccessType::Default) => expr.clone(),
            (VariableType::Scalar(expr), AccessType::Vector(idx)) => match expr {
                Expression::Elem(elem) => {
                    Expression::VectorAccess(VectorAccess::new(elem.clone(), idx))
                }
                Expression::VectorAccess(matrix_row_access) => {
                    Expression::MatrixAccess(MatrixAccess::new(
                        Identifier(matrix_row_access.name().to_string()),
                        matrix_row_access.idx(),
                        idx,
                    ))
                }
                _ => {
                    // TODO: replace this error
                    // return  Err(SemanticError::invalid_vector_access(
                    //     vector_access,
                    //     symbol_access.symbol().symbol_type(),
                    // ))
                    return Err(SemanticError::InvalidUsage(format!(
                        "Invalid variable access for variable type {variable_type:?}",
                    )));
                }
            },
            (
                VariableType::Scalar(Expression::Elem(elem)),
                AccessType::Matrix(row_idx, col_idx),
            ) => Expression::MatrixAccess(MatrixAccess::new(elem.clone(), row_idx, col_idx)),
            (VariableType::Vector(expr_vector), AccessType::Vector(idx)) => {
                expr_vector[idx].clone()
            }
            (VariableType::Vector(expr_vector), AccessType::Matrix(row_idx, col_idx)) => {
                match &expr_vector[row_idx] {
                    Expression::Elem(elem) => {
                        Expression::VectorAccess(VectorAccess::new(elem.clone(), col_idx))
                    }
                    Expression::VectorAccess(matrix_row_access) => {
                        Expression::MatrixAccess(MatrixAccess::new(
                            Identifier(matrix_row_access.name().to_string()),
                            matrix_row_access.idx(),
                            col_idx,
                        ))
                    }
                    _ =>
                    // TODO: replace this error
                    // Err(SemanticError::invalid_matrix_access(
                    //     matrix_access,
                    //     symbol_access.symbol().symbol_type(),
                    // )),
                    {
                        return Err(SemanticError::InvalidUsage(format!(
                            "Invalid variable access for variable type {variable_type:?}",
                        )))
                    }
                }
            }
            (VariableType::Matrix(expr_matrix), AccessType::Matrix(row_idx, col_idx)) => {
                expr_matrix[row_idx][col_idx].clone()
            }
            _ => {
                // TODO: update this error
                return Err(SemanticError::InvalidUsage(format!(
                    "Invalid variable access for variable type {variable_type:?}",
                )));
            }
        };

        Ok(expr)
    }
}

// struct VariableRoots {
//     roots: BTreeMap<SymbolAccess, ExprDetails>,
// }

// impl VariableRoots {

//     pub fn get_expr_details(&self, symbol_access: SymbolAccess, domain: ConstraintDomain) -> Option<ExprDetails> {
//         if let Some(expr_details) = self.roots.get(&symbol_access) {
//             // TODO: deal with boundary conflict properly
//             if expr_details.domain().is_boundary() {
//                 return Some(ExprDetails::new(expr_details.root_idx(), expr_details.trace_segment(), domain));
//             } else {
//                 return Some(*expr_details);
//             }
//         }

//         None
//     }

//     pub fn get_expr(&self,
//         symbol_access: SymbolAccess,
//         variable_type: &VariableType,
//         domain: ConstraintDomain,) -> Result<Expression, SemanticError> {
//             let expr = match (variable_type, symbol_access.access_type()) {
//                 (VariableType::Scalar(expr), AccessType::Default) => expr.clone(),
//                 (VariableType::Scalar(expr), AccessType::Vector(idx)) => match expr {
//                     Expression::Elem(elem) => {
//                         Expression::VectorAccess(VectorAccess::new(elem.clone(), *idx))
//                     }
//                     Expression::VectorAccess(matrix_row_access) => {
//                         Expression::MatrixAccess(MatrixAccess::new(
//                             Identifier(matrix_row_access.name().to_string()),
//                             matrix_row_access.idx(),
//                             *idx,
//                         ))
//                     }
//                     _ => {
//                         // TODO: replace this error
//                         // return  Err(SemanticError::invalid_vector_access(
//                         //     vector_access,
//                         //     symbol_access.symbol().symbol_type(),
//                         // ))
//                         return Err(SemanticError::InvalidUsage(format!(
//                             "Invalid variable access for variable type {variable_type:?} and symbol access {symbol_access:?}",
//                         )));
//                     }
//                 },
//                 (VariableType::Scalar(Expression::Elem(elem)), AccessType::Matrix(row_idx, col_idx)) => {
//                     Expression::MatrixAccess(MatrixAccess::new(
//                         elem.clone(),
//                         *row_idx,
//                         *col_idx,
//                     ))
//                 }
//                 (VariableType::Vector(expr_vector), AccessType::Vector(idx)) => expr_vector[*idx].clone(),
//                 (VariableType::Vector(expr_vector), AccessType::Matrix(row_idx, col_idx)) => match &expr_vector[*row_idx] {
//                     Expression::Elem(elem) => {
//                         Expression::VectorAccess(VectorAccess::new(elem.clone(), *col_idx))
//                     }
//                     Expression::VectorAccess(matrix_row_access) => {
//                         Expression::MatrixAccess(MatrixAccess::new(
//                             Identifier(matrix_row_access.name().to_string()),
//                             matrix_row_access.idx(),
//                             *col_idx,
//                         ))
//                     }
//                     _ =>
//                         // TODO: replace this error
//                         // Err(SemanticError::invalid_matrix_access(
//                         //     matrix_access,
//                         //     symbol_access.symbol().symbol_type(),
//                         // )),
//                         return Err(SemanticError::InvalidUsage(format!(
//                         "Invalid variable access for variable type {variable_type:?} and symbol access {symbol_access:?}",
//                     )))
//                 }
//                 (VariableType::Matrix(expr_matrix), AccessType::Matrix(row_idx, col_idx)) => {
//                     expr_matrix[*row_idx][*col_idx].clone()
//                 }
//                 _ => {
//                     // TODO: update this error
//                     return Err(SemanticError::InvalidUsage(format!(
//                         "Invalid variable access for variable type {variable_type:?} and symbol access {symbol_access:?}",
//                     )));
//                 }
//             };

//             Ok(expr)
//         }

//      /// Looks up the specified variable value in the variable roots and returns the expression
//     /// details if it is found. Otherwise, inserts the variable expression into the graph, adds it
//     /// to the variable roots, and returns the resulting expression details.
//     pub(super) fn insert_variable(
//         &mut self,
//         symbol_access: SymbolAccess,
//         variable_type: &VariableType,
//         domain: ConstraintDomain,
//     ) -> Result<ExprDetails, SemanticError> {
//         if let Some(expr_details) = self.roots.get(&symbol_access) {
//             // TODO: deal with boundary conflict properly
//             if expr_details.domain().is_boundary() {
//                 Ok(ExprDetails::new(expr_details.root_idx(), expr_details.trace_segment(), domain))
//             } else {
//                 Ok(*expr_details)
//             }
//         } else {
//             // Otherwise, insert the variable expression and create a new variable root.
//             let expr =self.get_expr(symbol_access, variable_type, domain)?;

//             let expr_details = self.insert_expr( &expr, domain)?;
//             self.roots.insert(symbol_access, expr_details);

//             Ok(expr_details)
//         }
//     }
// }
