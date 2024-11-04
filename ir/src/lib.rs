mod codegen;
mod graph;
mod ir;
mod ir2;
pub mod passes;
#[cfg(test)]
mod tests;

pub use self::codegen::CodeGenerator;
pub use self::graph::{MirGraph, Node, NodeIndex};
pub use self::ir::*;

use miden_diagnostics::{Diagnostic, ToDiagnostic};

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
/*
impl From<air_pass::Pass::Error> for CompileError {
    fn from(err: CompileError) -> Self {
        err.to_diagnostic()
    }
}*/

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
