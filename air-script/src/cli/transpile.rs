use clap::Args;
use std::{fs, path::PathBuf, sync::Arc};

use codegen_winter::CodeGenerator;
use ir::AirIR;
use miden_diagnostics::{
    term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
};
use parser::{ast::Source, Parser};

#[derive(Args)]
pub struct Transpile {
    /// Path to input file
    input: PathBuf,

    #[arg(
        short,
        long,
        help = "Output filename, default to the input file with the .rs extension"
    )]
    output: Option<PathBuf>,
}

impl Transpile {
    pub fn execute(&self) -> Result<(), String> {
        println!("============================================================");
        println!("Transpiling...");

        let input_path = &self.input;
        let output_path = match &self.output {
            Some(path) => path.clone(),
            None => {
                let mut path = input_path.clone();
                path.set_extension("rs");
                path
            }
        };

        let codemap = Arc::new(CodeMap::new());
        let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
        let diagnostics = DiagnosticsHandler::new(Default::default(), codemap.clone(), emitter);
        let parser = Parser::new((), codemap);

        // Parse from file to internal representation
        let parsed = match parser.parse_file::<Source, _, _>(&diagnostics, input_path) {
            Ok(ast) => ast,
            Err(err) => {
                diagnostics.emit(err);
                return Err("parsing failed".into());
            }
        };

        let ir = AirIR::new(parsed);
        if let Err(err) = ir {
            return Err(format!("{err:?}"));
        }
        let ir = ir.unwrap();

        // generate Rust code targeting Winterfell
        let codegen = CodeGenerator::new(&ir);

        // write transpiled output to the output path
        let result = fs::write(output_path.clone(), codegen.generate());
        if let Err(err) = result {
            return Err(format!("{err:?}"));
        }

        println!("Success! Transpiled to {}", output_path.display());
        println!("============================================================");

        Ok(())
    }
}
