mod access;
mod boundary_constraints;
mod buses;
mod constant;
mod evaluators;
mod integrity_constraints;
mod list_folding;
mod pub_inputs;
mod random_values;
mod selectors;
mod source_sections;
mod trace;
mod variables;

use std::sync::Arc;

use miden_diagnostics::{CodeMap, DiagnosticsConfig, DiagnosticsHandler, Verbosity};

pub use crate::CompileError;
use crate::compile;

pub fn compile_from_source(source: &str) -> Result<crate::Air, ()> {
    let compiler = Compiler::default();
    match compiler.compile(source) {
        Ok(air) => Ok(air),
        Err(err) => {
            compiler.diagnostics.emit(err);
            compiler.emitter.print_captured_to_stderr();
            Err(())
        },
    }
}

#[track_caller]
pub fn expect_diagnostic(source: &str, expected: &str) {
    let compiler = Compiler::default();
    let err = match compiler.compile(source) {
        Ok(ref ast) => {
            panic!("expected compilation to fail, got {ast:#?}");
        },
        Err(err) => err,
    };
    compiler.diagnostics.emit(err);
    let found = compiler.emitter.captured().contains(expected);
    if !found {
        compiler.emitter.print_captured_to_stderr();
    }
    assert!(found, "expected diagnostic output to contain the string: '{expected}'");
}

struct Compiler {
    codemap: Arc<CodeMap>,
    emitter: Arc<SplitEmitter>,
    diagnostics: Arc<DiagnosticsHandler>,
}
impl Default for Compiler {
    fn default() -> Self {
        Self::new(DiagnosticsConfig {
            verbosity: Verbosity::Warning,
            warnings_as_errors: true,
            no_warn: false,
            display: Default::default(),
        })
    }
}
impl Compiler {
    pub fn new(config: DiagnosticsConfig) -> Self {
        let codemap = Arc::new(CodeMap::new());
        let emitter = Arc::new(SplitEmitter::new());
        let diagnostics =
            Arc::new(DiagnosticsHandler::new(config, codemap.clone(), emitter.clone()));

        Self { codemap, emitter, diagnostics }
    }

    pub fn compile(&self, source: &str) -> Result<crate::Air, CompileError> {
        air_parser::parse(&self.diagnostics, self.codemap.clone(), source)
            .map_err(CompileError::Parse)
            .and_then(|program| compile(&self.diagnostics, program))
    }
}

struct SplitEmitter {
    capture: miden_diagnostics::CaptureEmitter,
    default: miden_diagnostics::DefaultEmitter,
}
impl SplitEmitter {
    #[inline]
    pub fn new() -> Self {
        use miden_diagnostics::term::termcolor::ColorChoice;

        Self {
            capture: Default::default(),
            default: miden_diagnostics::DefaultEmitter::new(ColorChoice::Auto),
        }
    }

    pub fn captured(&self) -> String {
        self.capture.captured()
    }

    pub fn print_captured_to_stderr(&self) {
        use std::io::Write;

        use miden_diagnostics::Emitter;

        let mut copy = self.default.buffer();
        let captured = self.capture.captured();
        copy.write_all(captured.as_bytes()).unwrap();
        self.default.print(copy).unwrap();
    }
}
impl miden_diagnostics::Emitter for SplitEmitter {
    #[inline]
    fn buffer(&self) -> miden_diagnostics::term::termcolor::Buffer {
        self.capture.buffer()
    }

    #[inline]
    fn print(&self, buffer: miden_diagnostics::term::termcolor::Buffer) -> std::io::Result<()> {
        self.capture.print(buffer)
    }
}
