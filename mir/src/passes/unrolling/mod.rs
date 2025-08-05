use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;

use super::visitor::Visitor;
use crate::{CompileError, ir::*};

mod match_optimizer;
mod unrolling_first_pass;
mod unrolling_second_pass;

use unrolling_first_pass::UnrollingFirstPass;
use unrolling_second_pass::UnrollingSecondPass;

/// This pass follows a similar approach as the Inlining pass and requires that the latter has
/// already been done.
///
/// * In the first step, we visit the graph, unrolling each node type except `For` nodes. Instead,
///   for these node types we gather the context to inline them in the second pass. In this first
///   pass, we also optimize constraints found in match statements.
/// * In the second pass, we inline the bodies of `For` nodes.
pub struct Unrolling<'a> {
    diagnostics: &'a DiagnosticsHandler,
}

impl<'a> Unrolling<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}

/// This structure is used to keep track of what is needed to inline a For node
#[derive(Clone, Debug)]
pub struct ForInliningContext {
    body: Link<Op>,
    iterators: Vec<Link<Op>>,
    selector: Option<Link<Op>>,
    ref_node: Link<Op>,
}

impl Pass for Unrolling<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        // The first pass unrolls all nodes fully, except for For nodes
        let mut first_pass = UnrollingFirstPass::new(self.diagnostics);
        Visitor::run(&mut first_pass, ir.constraint_graph_mut())?;

        // The second pass actually inlines the For nodes
        let mut second_pass = UnrollingSecondPass::new(
            self.diagnostics,
            first_pass.bodies_to_inline.clone(),
            first_pass.all_for_nodes.clone(),
        );
        Visitor::run(&mut second_pass, ir.constraint_graph_mut())?;
        Ok(ir)
    }
}
