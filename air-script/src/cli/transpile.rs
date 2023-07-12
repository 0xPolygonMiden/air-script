use std::{fs, path::PathBuf, sync::Arc};

use air_ir::{CodeGenerator, CompileError};
use air_pass::Pass;

use clap::{Args, ValueEnum};
use miden_diagnostics::{
    term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Target {
    Winterfell,
    Masm,
}
impl Target {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Winterfell => "rs",
            Self::Masm => "masm",
        }
    }
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
                let target = self.target.unwrap_or(Target::Winterfell);
                let backend: Box<dyn CodeGenerator<Output = String>> = match target {
                    Target::Winterfell => Box::new(air_codegen_winter::CodeGenerator),
                    Target::Masm => Box::<air_codegen_masm::CodeGenerator>::default(),
                };

                // write transpiled output to the output path
                let output_path = match &self.output {
                    Some(path) => path.clone(),
                    None => {
                        let mut path = input_path.clone();
                        path.set_extension(target.extension());
                        path
                    }
                };
                let code = backend.generate(&air).expect("code generation failed");
                if let Err(err) = fs::write(&output_path, code) {
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
