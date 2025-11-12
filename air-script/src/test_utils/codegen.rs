use std::sync::Arc;

use crate::compile;
use air_ir::{CodeGenerator, CompileError};
use miden_diagnostics::{
    CodeMap, DefaultEmitter, DiagnosticsHandler, term::termcolor::ColorChoice,
};

pub enum Target {
    Winterfell,
    Plonky3,
}

pub struct Test {
    input_path: String,
}
impl Test {
    pub fn new(input_path: String) -> Self {
        Test { input_path }
    }

    pub fn transpile(&self, target: Target) -> Result<String, CompileError> {
        let codemap = Arc::new(CodeMap::new());
        let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
        let diagnostics = DiagnosticsHandler::new(Default::default(), codemap.clone(), emitter);

        // Parse from file to internal representation
        let air = air_parser::parse_file(&diagnostics, codemap, &self.input_path)
            .map_err(CompileError::Parse)
            .and_then(|program| compile(&diagnostics, program))?;

        let backend: Box<dyn CodeGenerator<Output = String>> = match target {
            Target::Winterfell => Box::new(air_codegen_winter::CodeGenerator),
            Target::Plonky3 => Box::new(air_codegen_plonky3::CodeGenerator),
        };

        // generate Rust code targeting Winterfell
        Ok(backend.generate(&air).expect("code generation failed"))
    }
}
