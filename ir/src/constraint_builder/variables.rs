use super::{
    AccessType, ConstraintBuilder, Expression, Identifier, MatrixAccess, SemanticError, Symbol,
    ValidateAccess, VariableType, VectorAccess,
};

impl ConstraintBuilder {
    pub fn get_variable_expr(
        &self,
        symbol: &Symbol,
        access_type: &AccessType,
        variable_type: &VariableType,
    ) -> Result<Expression, SemanticError> {
        variable_type.validate(symbol.name(), access_type)?;

        let expr = match (variable_type, access_type) {
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
                    return Err(SemanticError::invalid_variable_access_type(
                        symbol.name(),
                        access_type,
                    ))
                }
            },
            (
                VariableType::Scalar(Expression::Elem(elem)),
                AccessType::Matrix(row_idx, col_idx),
            ) => Expression::MatrixAccess(MatrixAccess::new(elem.clone(), *row_idx, *col_idx)),
            (VariableType::Vector(expr_vector), AccessType::Vector(idx)) => {
                expr_vector[*idx].clone()
            }
            (VariableType::Vector(expr_vector), AccessType::Matrix(row_idx, col_idx)) => {
                match &expr_vector[*row_idx] {
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
                    _ => {
                        return Err(SemanticError::invalid_variable_access_type(
                            symbol.name(),
                            access_type,
                        ))
                    }
                }
            }
            (VariableType::Matrix(expr_matrix), AccessType::Matrix(row_idx, col_idx)) => {
                expr_matrix[*row_idx][*col_idx].clone()
            }
            _ => {
                return Err(SemanticError::invalid_variable_access_type(
                    symbol.name(),
                    access_type,
                ))
            }
        };

        Ok(expr)
    }
}
