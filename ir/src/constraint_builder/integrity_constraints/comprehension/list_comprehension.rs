use super::{
    build_iterable_context, ConstraintBuilder, Expression, ListComprehension, SemanticError,
};

impl ConstraintBuilder {
    /// Unfolds a list comprehension into a vector of expressions.
    ///
    /// # Errors
    /// - Returns an error if there is an error while parsing any of the expressions in the expanded
    /// vector from the list comprehension.
    pub fn unfold_lc(&self, lc: &ListComprehension) -> Result<Vec<Expression>, SemanticError> {
        let num_iterations = self.get_num_iterations(lc.context())?;
        if num_iterations == 0 {
            return Err(SemanticError::InvalidComprehension(
                "List comprehensions must have at least one iteration.".to_string(),
            ));
        }

        let iterable_context = build_iterable_context(&lc.context().to_vec())?;
        let vector = (0..num_iterations)
            .map(|i| self.parse_comprehension_expr(lc.expression(), &iterable_context, i))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(vector)
    }
}
