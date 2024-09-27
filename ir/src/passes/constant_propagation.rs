use std::ops::ControlFlow;

use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;

use crate::MirGraph;

pub struct ConstantPropagation<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,
}
impl<'p> Pass for ConstantPropagation<'p> {
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

impl<'a> ConstantPropagation<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }

    fn run_visitor(&mut self, ir: &mut MirGraph) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
