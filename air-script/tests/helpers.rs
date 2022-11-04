use codegen_winter::CodeGenerator;
use ir::AirIR;
use parser::parse;
use std::fs;

#[derive(Debug)]
pub enum TestError {
    IO(String),
    Parse(String),
    IR(String),
}

pub struct Test {
    input_path: String,
}

impl Test {
    pub fn new(input_path: String) -> Self {
        Test { input_path }
    }

    pub fn transpile(&self) -> Result<String, TestError> {
        // load source input from file
        let source = fs::read_to_string(&self.input_path).map_err(|err| {
            TestError::IO(format!(
                "Failed to open input file `{:?}` - {}",
                self.input_path, err
            ))
        })?;

        // parse the input file to the internal representation
        let parsed = parse(source.as_str()).map_err(|_| {
            TestError::Parse(format!(
                "Failed to parse the input air file at {}",
                &self.input_path
            ))
        })?;

        let ir = AirIR::from_source(&parsed).map_err(|_| {
            TestError::IR(format!(
                "Failed to convert the input air file at {} to IR representation",
                &self.input_path
            ))
        })?;

        // generate Rust code targeting Winterfell
        let codegen = CodeGenerator::new(&ir);
        Ok(codegen.generate())
    }
}
