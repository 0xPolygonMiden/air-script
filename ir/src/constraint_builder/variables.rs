use super::{
    AccessType, ConstraintBuilder, Expression, Identifier, MatrixAccess, SemanticError, Symbol,
    VariableType, VectorAccess,
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
