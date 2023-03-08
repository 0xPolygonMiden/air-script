use std::{fs, path::PathBuf};
use structopt::StructOpt;

use codegen_winter::CodeGenerator;
use ir::AirIR;
use parser::parse;

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

        // load source input from file
        let source = fs::read_to_string(input_path).map_err(|err| {
            format!(
                "Failed to open input file `{:?}` - {}",
                &self.input_file, err
            )
        })?;

        // parse the input file to the internal representation
        let parsed = parse(source.as_str());
        if let Err(err) = parsed {
            return Err(format!("{err:?}"));
        }
        let parsed = parsed.unwrap();

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
