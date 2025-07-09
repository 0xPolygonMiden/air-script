use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;

use crate::{Air, CompileError};

pub struct CommonSubexpressionElimination<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,
}

impl Pass for CommonSubexpressionElimination<'_> {
    type Input<'a> = Air;
    type Output<'a> = Air;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        // 1. Start by going through all the nodes in the Air and evaluating them at random points.
        let mut rng = rand::rng();
        let evals = ir.constraint_graph_mut().evaluate_all_nodes(&mut rng)?;

        // 2. Then, eliminate common subexpressions in the graph based on the evaluations.
        // This will both:
        // - Remove nodes that have the same evaluation, keeping only one instance.
        // - Update the indices of the nodes to reflect the changes in the graph.
        let renumbering_map = ir.constraint_graph_mut().eliminate_common_subexpressions(&evals);

        // Update constraints with the new node indices
        ir.constraints.renumber_constraints(&renumbering_map);

        Ok(ir)
    }
}

impl<'a> CommonSubexpressionElimination<'a> {
    #[allow(unused)]
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}
