pub mod ir;
pub mod passes;
#[cfg(test)]
mod tests;

use air_parser::ast::Program;
use air_pass::Pass;
use miden_diagnostics::{Diagnostic, DiagnosticsHandler, ToDiagnostic};

use crate::ir::Mir;

/// Abstracts the various passes done on the MIR representation of the program.
pub struct MirPasses<'a> {
    diagnostics: &'a DiagnosticsHandler,
}

impl<'a> MirPasses<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}

impl Pass for MirPasses<'_> {
    type Input<'a> = Program;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, input: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut ast_to_mir = passes::AstToMir::new(self.diagnostics);
        let mir = ast_to_mir.run(input)?;

        let mut projection = passes::IndexProjection::new(self.diagnostics);
        let mir = projection.run(mir)?;

        let mut inlining = passes::Inlining::new(self.diagnostics);
        let mir = inlining.run(mir)?;

        // AIR_DISABLE_CSE disables the common subexpression elimination pass.
        let mir = if std::env::var("AIR_DISABLE_CSE").is_ok() {
            mir
        } else {
            let mut cse = passes::Cse::new();
            let mir = cse.run(mir)?;
            mir
        };

        let mut unrolling = passes::Unrolling::new(self.diagnostics);
        let mir = unrolling.run(mir)?;

        let mut constant_prop = passes::ConstantPropagation::new(self.diagnostics);
        let mir = constant_prop.run(mir)?;

        Ok(mir)
    }
}

/// Error type that can be returned during the Mir passes
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error(transparent)]
    Parse(#[from] air_parser::ParseError),
    #[error(transparent)]
    SemanticAnalysis(#[from] air_parser::SemanticAnalysisError),
    #[error("compilation failed, see diagnostics for more information")]
    Failed,
}

impl ToDiagnostic for CompileError {
    /// Helper to convert a [CompileError] into a [Diagnostic]
    fn to_diagnostic(self) -> Diagnostic {
        match self {
            Self::Parse(err) => err.to_diagnostic(),
            Self::SemanticAnalysis(err) => err.to_diagnostic(),
            Self::Failed => Diagnostic::error().with_message(self.to_string()),
        }
    }
}
