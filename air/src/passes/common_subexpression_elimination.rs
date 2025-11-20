use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;

use crate::{Air, CompileError};

/// This pass aims to remove duplicate nodes in the Algebraic Graph by evaluating
/// each node at random inputs. The process relies on:
/// - Iterating over and evaluating all the nodes in order of their NodeIndex
/// - Do not insert nodes that evaluate to the same value as an existing node
/// - Update the indices of the nodes in the graph to reflect the changes
///
/// Note: This pass requires that boundary constraint should be inserted in the graph before
/// integrity constraints or buses, to keep the nodes consistent with Winterfell codegen's
/// expectation.
pub struct CommonSubexpressionElimination<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,
}

impl Pass for CommonSubexpressionElimination<'_> {
    type Input<'a> = Air;
    type Output<'a> = Air;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        // Evaluate the nodes at random points, and eliminate common subexpressions in the graph
        // based on the evaluations. This will both:
        // - Remove nodes that have the same evaluation, keeping only one instance.
        // - Update the indices of the nodes to reflect the changes in the graph.
        let renumbering_map = ir.constraint_graph_mut().eliminate_common_subexpressions();

        // Update constraints with the new node indices
        ir.constraints.renumber_and_deduplicate_constraints(&renumbering_map);

        // Iterate over all bus transition expression and renumber their node indices
        for (_, value) in ir.buses_initial_values.iter_mut() {
            let new_value_index = *renumbering_map
                .get(value)
                .expect("Error: cannot find value index in renumbering map");
            *value = new_value_index;
        }

        // Iterate over all bus transition expression and renumber their node indices
        for (_, (numerator, denominator)) in ir.buses_transitions.iter_mut() {
            let new_numerator_index = *renumbering_map
                .get(numerator)
                .expect("Error: cannot find numerator index in renumbering map");
            *numerator = new_numerator_index;
            if let Some(denominator) = denominator {
                let new_denominator_index = *renumbering_map
                    .get(denominator)
                    .expect("Error: cannot find denominator index in renumbering map");
                *denominator = new_denominator_index;
            }
        }

        Ok(ir)
    }
}

impl<'a> CommonSubexpressionElimination<'a> {
    #[allow(unused)]
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}
