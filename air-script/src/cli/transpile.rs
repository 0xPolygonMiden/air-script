use clap::{Args, ValueEnum};
use std::{fs, path::PathBuf, sync::Arc};

use ir::AirIR;
use miden_diagnostics::{
    term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
};
use parser::{ast::Source, Parser};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Target {
    Winterfell,
    Masm,
}

#[derive(Args)]
pub struct Transpile {
    /// Path to input file
    input: PathBuf,

    #[arg(
        short,
        long,
        help = "Output filename, defaults to the input file with the .rs extension for Winterfell or .masm for MASM"
    )]
    output: Option<PathBuf>,

    #[arg(
        short,
        long,
        help = "Defines the target language, defaults to Winterfell"
    )]
    target: Option<Target>,
}

impl Transpile {
    pub fn execute(&self) -> Result<(), String> {
        println!("============================================================");
        println!("Transpiling...");

        let input_path = &self.input;

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

        let (code, extension) = match self.target.unwrap_or(Target::Winterfell) {
            Target::Winterfell => (codegen_winter::CodeGenerator::new(&ir).generate(), "rs"),
            Target::Masm => (
                codegen_masm::CodeGenerator::new(&ir, codegen_masm::CodegenConfig::default())
                    .generate()
                    .expect("code generation failed"),
                "masm",
            ),
        };

        let output_path = match &self.output {
            Some(path) => path.clone(),
            None => {
                let mut path = input_path.clone();
                path.set_extension(extension);
                path
            }
        };

        // write transpiled output to the output path
        let result = fs::write(output_path.clone(), code);
        if let Err(err) = result {
            return Err(format!("{err:?}"));
        }

        println!("Success! Transpiled to {}", output_path.display());
        println!("============================================================");

        Ok(())
    }
}
