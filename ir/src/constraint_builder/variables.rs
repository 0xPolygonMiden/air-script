use super::{
    AccessType, Expression, Identifier, MatrixAccess, SemanticError, ValidateAccess,
    VariableValueExpr, VectorAccess,
};

/// TODO: add doc comment and add comments in the code to explain the logic
pub(crate) fn get_variable_expr(
    variable_name: &str,
    variable_type: &VariableValueExpr,
    access_type: &AccessType,
) -> Result<Expression, SemanticError> {
    variable_type.validate(variable_name, access_type)?;

    let expr = match (variable_type, access_type) {
        (VariableValueExpr::Scalar(expr), AccessType::Default) => expr.clone(),
        (VariableValueExpr::Scalar(expr), AccessType::Vector(idx)) => match expr {
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
                    variable_name,
                    access_type,
                ))
            }
        },
        (
            VariableValueExpr::Scalar(Expression::Elem(elem)),
            AccessType::Matrix(row_idx, col_idx),
        ) => Expression::MatrixAccess(MatrixAccess::new(elem.clone(), *row_idx, *col_idx)),
        (VariableValueExpr::Vector(expr_vector), AccessType::Vector(idx)) => {
            expr_vector[*idx].clone()
        }
        (VariableValueExpr::Vector(expr_vector), AccessType::Matrix(row_idx, col_idx)) => {
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
                        variable_name,
                        access_type,
                    ))
                }
            }
        }
        (VariableValueExpr::Matrix(expr_matrix), AccessType::Matrix(row_idx, col_idx)) => {
            expr_matrix[*row_idx][*col_idx].clone()
        }
        _ => {
            return Err(SemanticError::invalid_variable_access_type(
                variable_name,
                access_type,
            ))
        }
    };

    Ok(expr)
}
