use std::{fs, path::PathBuf, sync::Arc};

use codegen_winter::CodeGenerator;
use ir::AirIR;
use miden_diagnostics::{
    term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
};
use parser::{ast::Source, Parser};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "Transpile",
    about = "Transpile AirScript source code to Rust targeting Winterfell"
)]
pub struct TranspileCmd {
    /// Path to input file
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    input_file: Option<PathBuf>,
    /// Path to output file
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output_file: Option<PathBuf>,
}

impl TranspileCmd {
    pub fn execute(&self) -> Result<(), String> {
        println!("============================================================");
        println!("Transpiling...");

        // get the input path
        let input_path = match &self.input_file {
            Some(path) => path.clone(),
            None => {
                return Err("No input file specified".to_string());
            }
        };

        // get the output path
        let output_path = match &self.output_file {
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
