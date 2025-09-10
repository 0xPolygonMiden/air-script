mod codegen;
mod graph;
mod ir;
pub mod passes;
#[cfg(test)]
mod tests;

use air_parser::ast::Program;
use air_pass::Pass;
use miden_diagnostics::{Diagnostic, DiagnosticsHandler, ToDiagnostic};
use mir::ir::Mir;

pub use self::{
    codegen::CodeGenerator,
    graph::{AlgebraicGraph, Node, NodeIndex},
    ir::*,
};

/// Compiles an AirScript program from the the parsed AST to the AIR
pub fn compile(diagnostics: &DiagnosticsHandler, program: Program) -> Result<Air, CompileError> {
    let mut pipeline = ast_to_air_pipeline(diagnostics);
    pipeline.run(program)
}

/// Creates a pipeline of passes that transforms an AST into AIR.
pub fn ast_to_air_pipeline<'a>(
    diagnostics: &DiagnosticsHandler,
) -> impl Pass<Input<'a> = Program, Output<'a> = Air, Error = CompileError> {
    let ast_passes = air_parser::AstPasses::new(diagnostics);
    let mir_passes = mir::MirPasses::new(diagnostics);
    let air_ir_passes = crate::AirPasses::new(diagnostics);

    ast_passes.chain(mir_passes).chain(air_ir_passes)
}

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
            .chain(passes::BusOpExpand::new(self.diagnostics))
            .chain(passes::CommonSubexpressionElimination::new(self.diagnostics));
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
