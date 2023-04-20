use super::{
    AccessType, ConstantValueExpr, ConstraintBuilder, Expression, ListFoldingValueExpr,
    SemanticError, SymbolAccess, SymbolBinding, VariableValueExpr, CURRENT_ROW,
};

// LIST FOLDING
// ================================================================================================

impl ConstraintBuilder {
    /// Builds a list of expressions from a list folding value. The list folding value can be either a
    /// vector, a list comprehension, or an identifier that refers to a vector.
    ///
    /// # Errors
    /// Returns an error if:
    /// - the list folding value is an identifier that does not exist in the symbol table
    /// - the list folding value is an identifier that does not refer to a vector
    pub fn build_list_from_list_folding_value(
        &self,
        lf_value_type: &ListFoldingValueExpr,
    ) -> Result<Vec<Expression>, SemanticError> {
        match lf_value_type {
            ListFoldingValueExpr::Identifier(ident) => {
                let symbol = self.symbol_table.get_symbol(ident.name())?;
                match symbol.binding() {
                    SymbolBinding::Constant(ConstantValueExpr::Vector(list)) => {
                        Ok(list.iter().map(|value| Expression::Const(*value)).collect())
                    }
                    SymbolBinding::Variable(variable_type) => {
                        if let VariableValueExpr::Vector(list) = variable_type {
                            Ok(list.clone())
                        } else {
                            Err(SemanticError::invalid_list_folding(
                                lf_value_type,
                                symbol.binding(),
                            ))
                        }
                    }
                    SymbolBinding::Trace(columns) => {
                        if columns.size() > 1 {
                            Ok((0..columns.size())
                                .map(|i| {
                                    Expression::SymbolAccess(SymbolAccess::new(
                                        ident.clone(),
                                        AccessType::Vector(i),
                                        CURRENT_ROW,
                                    ))
                                })
                                .collect())
                        } else {
                            Err(SemanticError::invalid_list_folding(
                                lf_value_type,
                                symbol.binding(),
                            ))
                        }
                    }
                    _ => Err(SemanticError::invalid_list_folding(
                        lf_value_type,
                        symbol.binding(),
                    )),
                }
            }
            ListFoldingValueExpr::Vector(vector) => Ok(vector.clone()),
            ListFoldingValueExpr::ListComprehension(lc) => Ok(self.unfold_lc(lc)?),
        }
    }
}
