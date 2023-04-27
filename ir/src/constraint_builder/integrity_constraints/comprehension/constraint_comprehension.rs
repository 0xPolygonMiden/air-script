use air_script_core::ComprehensionContext;
use parser::ast::{ConstraintExpr, InlineConstraintExpr};

use super::{build_iterable_context, ComprehensionType, ConstraintBuilder, SemanticError};

impl ConstraintBuilder {
    /// Unfolds a constraint comprehension into a vector of expressions.
    ///
    /// # Errors
    /// - Returns an error if there is an error while parsing any of the expressions in the expanded
    /// vector from the constraint comprehension.
    pub fn unfold_cc(
        &self,
        constraint: &ConstraintExpr,
        cc_context: &ComprehensionContext,
    ) -> Result<Vec<ConstraintExpr>, SemanticError> {
        let num_iterations = self.get_num_iterations(&ComprehensionType::Constraint, cc_context)?;
        if num_iterations == 0 {
            return Err(SemanticError::InvalidComprehension(
                "Constraint comprehensions must have at least one iteration.".to_string(),
            ));
        }

        let iterable_context = build_iterable_context(&ComprehensionType::Constraint, cc_context)?;
        let mut constraints = Vec::new();
        for i in 0..num_iterations {
            match constraint {
                ConstraintExpr::Inline(inline_constraint) => {
                    let lhs = self.parse_comprehension_expr(
                        &ComprehensionType::Constraint,
                        inline_constraint.lhs(),
                        &iterable_context,
                        i,
                    )?;
                    let rhs = self.parse_comprehension_expr(
                        &ComprehensionType::Constraint,
                        inline_constraint.rhs(),
                        &iterable_context,
                        i,
                    )?;
                    let new_constraint =
                        ConstraintExpr::Inline(InlineConstraintExpr::new(lhs, rhs));
                    constraints.push(new_constraint);
                }
                ConstraintExpr::Evaluator(_) => {
                    todo!()
                }
            }
        }
        Ok(constraints)
    }
}
