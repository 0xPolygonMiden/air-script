use std::{fs, path::PathBuf, sync::Arc};

use air_ir::CompileError;
use air_pass::Pass;

use clap::Args;
use codegen_winter::CodeGenerator;
use miden_diagnostics::{
    term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
};

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

        // Parse from file to internal representation
        let air = air_parser::parse_file(&diagnostics, codemap, input_path)
            .map_err(CompileError::Parse)
            .and_then(|ast| {
                let mut pipeline = air_parser::transforms::ConstantPropagation::new(&diagnostics)
                    .chain(air_parser::transforms::Inlining::new(&diagnostics))
                    .chain(air_ir::passes::AstToAir::new(&diagnostics));
                pipeline.run(ast)
            });

        match air {
            Ok(air) => {
                // generate Rust code targeting Winterfell
                let codegen = CodeGenerator::new(&air);

                // write transpiled output to the output path
                let result = fs::write(output_path.clone(), codegen.generate());
                if let Err(err) = result {
                    return Err(format!("{err:?}"));
                }

                println!("Success! Transpiled to {}", output_path.display());
                println!("============================================================");

                Ok(())
            }
            Err(err) => {
                diagnostics.emit(err);
                Err("compilation failed".into())
            }
        }
    }
}
