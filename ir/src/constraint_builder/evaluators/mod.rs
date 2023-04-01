use super::{
    ast::IntegrityStmt, AlgebraicGraph, ConstraintBuilder, ConstraintBuilderContext, NodeIndex,
    SemanticError, TraceAccess, TraceBinding, Value,
};

mod evaluator;
pub(crate) use evaluator::Evaluator;

impl ConstraintBuilder {
    pub(crate) fn into_evaluator(self) -> Result<Evaluator, SemanticError> {
        if let ConstraintBuilderContext::EvaluatorFunction(params) = self.context {
            let evaluator = Evaluator::new(
                params,
                self.param_nodes,
                self.integrity_constraints,
                self.graph,
            );

            Ok(evaluator)
        } else {
            Err(SemanticError::invalid_context(
                "EvaluatorFunction",
                self.context,
            ))
        }
    }

    /// Processes an evaluator function with the given parameters and integrity statements.
    pub(crate) fn process_evaluator(
        &mut self,
        params: Vec<TraceBinding>,
        integrity_stmts: Vec<IntegrityStmt>,
    ) -> Result<(), SemanticError> {
        self.context = ConstraintBuilderContext::EvaluatorFunction(params.clone());

        // add evaluator parameters to the symbol table
        self.symbol_table.insert_ev_parameters(params)?;

        for stmt in integrity_stmts {
            self.process_integrity_stmt(stmt)?;
        }

        Ok(())
    }
}
