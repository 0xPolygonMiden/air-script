use codegen_gce::GCECodeGenerator;
use codegen_winter::WinterfellCodeGenerator;
use ir::AirIR;
use parser::parse;
use std::fs;

#[derive(Debug)]
pub enum TestError {
    IO(String),
    Parse(String),
    IR(String),
    Gce(String),
}

pub struct Test {
    ir: AirIR,
}

impl Test {
    pub fn new(input_path: String) -> Result<Self, TestError> {
        // load source input from file
        let source = fs::read_to_string(&input_path).map_err(|err| {
            TestError::IO(format!(
                "Failed to open input file `{:?}` - {}",
                input_path, err
            ))
        })?;

        // parse the input file to the internal representation
        let parsed = parse(source.as_str()).map_err(|_| {
            TestError::Parse(format!(
                "Failed to parse the input air file at {}",
                input_path
            ))
        })?;

        let ir = AirIR::from_source(&parsed).map_err(|_| {
            TestError::IR(format!(
                "Failed to convert the input air file at {} to IR representation",
                input_path
            ))
        })?;

        Ok(Test { ir })
    }

    pub fn generate_winterfell(&self) -> String {
        // generate Rust code targeting Winterfell
        let codegen = WinterfellCodeGenerator::new(&self.ir);
        codegen.generate()
    }

    pub fn generate_gce(&self, extension_degree: u8, path: &str) -> Result<(), TestError> {
        // generate Rust code targeting Winterfell
        let codegen = GCECodeGenerator::new(&self.ir, extension_degree).map_err(|err| {
            TestError::Gce(format!("Failed to create GCECodeGenerator: {:?}", err))
        })?;
        codegen
            .generate(path)
            .map_err(|err| TestError::Gce(format!("Failed to generate JSON file: {:?}", err)))
    }
}
