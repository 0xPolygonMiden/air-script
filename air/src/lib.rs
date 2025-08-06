mod codegen;
mod graph;
mod ir;
pub mod passes;
#[cfg(test)]
mod tests;

use miden_diagnostics::{Diagnostic, ToDiagnostic};
use mir::ir::Mir;

pub use self::{
    codegen::CodeGenerator,
    graph::{AlgebraicGraph, Node, NodeIndex},
    ir::*,
};

/// Abstracts the various passes done on the AIR representation of the program.
pub struct AirPasses<'a> {
    diagnostics: &'a DiagnosticsHandler,
}

impl<'a> AirPasses<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}

impl Pass for AirPasses<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Air;
    type Error = CompileError;

    fn run<'a>(&mut self, input: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut passes = passes::MirToAir::new(self.diagnostics)
            .chain(passes::BusOpExpand::new(self.diagnostics));
        passes.run(input)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error(transparent)]
    Parse(#[from] air_parser::ParseError),
    #[error(transparent)]
    SemanticAnalysis(#[from] air_parser::SemanticAnalysisError),
    #[error(transparent)]
    InvalidConstraint(#[from] ConstraintError),
    #[error("compilation failed, see diagnostics for more information")]
    Failed,
}

impl From<mir::CompileError> for CompileError {
    fn from(err: mir::CompileError) -> Self {
        match err {
            mir::CompileError::Parse(err) => Self::Parse(err),
            mir::CompileError::SemanticAnalysis(err) => Self::SemanticAnalysis(err),
            mir::CompileError::Failed => Self::Failed,
        }
    }
}

impl ToDiagnostic for CompileError {
    fn to_diagnostic(self) -> Diagnostic {
        match self {
            Self::Parse(err) => err.to_diagnostic(),
            Self::SemanticAnalysis(err) => err.to_diagnostic(),
            Self::InvalidConstraint(err) => Diagnostic::error().with_message(err.to_string()),
            Self::Failed => Diagnostic::error().with_message(self.to_string()),
        }
    }
}
