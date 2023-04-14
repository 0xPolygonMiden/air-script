use super::{
    ast::EvaluatorFunction, AlgebraicGraph, BTreeMap, ConstraintBuilder, ConstraintRoot,
    SemanticError,
};

impl ConstraintBuilder {
    /// TODO: docs
    pub(crate) fn process_evaluator(
        mut self,
        ev_decl: EvaluatorFunction,
    ) -> Result<BTreeMap<String, Evaluator>, SemanticError> {
        let (name, trace_params, integrity_stmts) = ev_decl.into_parts();

        // insert all of the parameters as trace bindings
        self.symbol_table.insert_trace_bindings(trace_params)?;

        // process all of the integrity variables and costraints
        for stmt in integrity_stmts {
            self.process_integrity_stmt(stmt)?;
        }

        let mut evaluators = self.evaluators;
        let evaluator = Evaluator::new(self.graph, self.integrity_constraints);
        evaluators.insert(name, evaluator);

        Ok(evaluators)
    }
}

// EVALUATOR FUNCTION STRUCT
// ================================================================================================

/// TODO
#[derive(Default, Debug, Clone)]
pub(crate) struct Evaluator {
    /// A list of root indices for each constraint defined in this evaluator function.
    constraints: Vec<Vec<ConstraintRoot>>,

    /// A directed acyclic graph which represents all of the constraints defined in this evaluator
    /// and their subexpressions.
    graph: AlgebraicGraph,
}

impl Evaluator {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    pub fn new(graph: AlgebraicGraph, constraints: Vec<Vec<ConstraintRoot>>) -> Self {
        Self { constraints, graph }
    }
}
