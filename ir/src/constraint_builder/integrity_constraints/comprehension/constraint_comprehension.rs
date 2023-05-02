use super::{
    build_iterable_context, ComprehensionContext, ConstraintBuilder, ConstraintExpr,
    EvaluatorFunctionCall, Expression, Identifier, InlineConstraintExpr, NodeIndex, SemanticError,
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
            ConstraintExpr::Evaluator(ev_call) => {
                for i in 0..num_iterations {
                    let mut symbols = Vec::new();
                    for segment in ev_call.args() {
                        let mut segment_symbols = Vec::new();
                        for arg in segment {
                            let arg_parsed = self.parse_symbol_access(arg, &iterable_context, i)?;
                            let arg_symbol = if let Expression::SymbolAccess(symbol) = arg_parsed {
                                symbol
                            } else {
                                return Err(SemanticError::invalid_evaluator_args(
                                    ev_call.name(),
                                    arg_parsed,
                                ))?;
                            };
                            segment_symbols.push(arg_symbol);
                        }
                        symbols.push(segment_symbols);
                    }
                    let ev_call_parsed =
                        EvaluatorFunctionCall::new(Identifier(ev_call.name().to_string()), symbols);
                    self.process_evaluator_call(ev_call_parsed, selectors)?;
                }
            }
        }
        Ok(())
    }
}
