use std::ops::ControlFlow;

use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;

use crate::MirGraph;

pub struct Inlining<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,
}
impl<'p> Pass for Inlining<'p> {
    type Input<'a> = MirGraph;
    type Output<'a> = MirGraph;
    type Error = ();

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        match self.run_visitor(&mut ir) {
            ControlFlow::Continue(()) => Ok(ir),
            ControlFlow::Break(err) => Err(err),
        }
    }
}

impl<'a> Inlining<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }

    //TODO MIR: Implement inlining pass on MIR
    // 1. Understand the basics of the previous inlining process
    // 2. Remove what is done during lowering from AST to MIR (unroll, ...)
    // 3. Check how it translates to the MIR structure
    fn run_visitor(&mut self, _ir: &mut MirGraph) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
