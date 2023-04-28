use parser::ast::{ConstraintExpr, InlineConstraintExpr};

use super::{
    build_iterable_context, ComprehensionContext, ConstraintBuilder, NodeIndex, SemanticError,
};

impl ConstraintBuilder {
    /// Processes a constraint comprehension. The constraint comprehension is expanded into
    /// constraints for each iteration of the comprehension and then processed.
    ///
    /// # Errors
    /// - Returns an error if there is an error while parsing any of the expressions in the expanded
    /// vector from the constraint comprehension.
    pub fn process_cc(
        &mut self,
        constraint: &ConstraintExpr,
        cc_context: &ComprehensionContext,
        selectors: Option<NodeIndex>,
    ) -> Result<(), SemanticError> {
        let num_iterations = self.get_num_iterations(cc_context)?;
        if num_iterations == 0 {
            return Err(SemanticError::InvalidComprehension(
                "Constraint comprehensions must have at least one iteration.".to_string(),
            ));
        }

        let iterable_context = build_iterable_context(cc_context)?;
        match constraint {
            ConstraintExpr::Inline(inline_constraint) => {
                for i in 0..num_iterations {
                    let lhs = self.parse_comprehension_expr(
                        inline_constraint.lhs(),
                        &iterable_context,
                        i,
                    )?;
                    let rhs = self.parse_comprehension_expr(
                        inline_constraint.rhs(),
                        &iterable_context,
                        i,
                    )?;

                    self.process_integrity_constraint(
                        InlineConstraintExpr::new(lhs, rhs),
                        selectors,
                    )?;
                }
            }
            ConstraintExpr::Evaluator(_) => {
                todo!()
            }
        }
        Ok(())
    }
}
