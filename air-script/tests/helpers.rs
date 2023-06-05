use std::sync::Arc;

use air_ir::CompileError;
use air_pass::Pass;
use codegen_winter::CodeGenerator;
use miden_diagnostics::{
    term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
};

pub struct Test {
    input_path: String,
}
impl Test {
    pub fn new(input_path: String) -> Self {
        Test { input_path }
    }

    pub fn transpile(&self) -> Result<String, CompileError> {
        let codemap = Arc::new(CodeMap::new());
        let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
        let diagnostics = DiagnosticsHandler::new(Default::default(), codemap.clone(), emitter);

        // Parse from file to internal representation
        let air = air_parser::parse_file(&diagnostics, codemap, &self.input_path)
            .map_err(CompileError::Parse)
            .and_then(|ast| {
                let mut pipeline = air_parser::transforms::ConstantPropagation::new(&diagnostics)
                    .chain(air_parser::transforms::Inlining::new(&diagnostics))
                    .chain(air_ir::passes::AstToAir::new(&diagnostics));
                pipeline.run(ast)
            })?;

        // generate Rust code targeting Winterfell
        let codegen = CodeGenerator::new(&air);
        Ok(codegen.generate())
    }
}
