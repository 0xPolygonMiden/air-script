use air_script::{parse, AirIR, GceCodeGenerator, WinterfellCodeGenerator};
use std::fs;

#[derive(Debug)]
pub enum TestError {
    IO(String),
    Parse(String),
    IR(String),
    Codegen(String),
}

pub struct Test {
    input_path: String,
}

impl Test {
    pub fn new(input_path: String) -> Self {
        Test { input_path }
    }

    /// Parse data in file at `input_path` and return [AirIR] with this data
    fn generate_ir(&self) -> Result<AirIR, TestError> {
        // load source input from file
        let source = fs::read_to_string(&self.input_path).map_err(|err| {
            TestError::IO(format!(
                "Failed to open input file `{:?}` - {}",
                &self.input_path, err
            ))
        })?;

        // parse the input file to the internal representation
        let parsed = parse(source.as_str()).map_err(|_| {
            TestError::Parse(format!(
                "Failed to parse the input air file at {}",
                &self.input_path
            ))
        })?;

        let ir = AirIR::new(&parsed).map_err(|_| {
            TestError::IR(format!(
                "Failed to convert the input air file at {} to IR representation",
                &self.input_path
            ))
        })?;

        Ok(ir)
    }

    /// Generate Rust code containing a Winterfell Air implementation for the AirIR
    pub fn generate_winterfell(&self) -> Result<String, TestError> {
        let ir = Self::generate_ir(self)?;
        // generate Rust code targeting Winterfell
        let codegen = WinterfellCodeGenerator::new(&ir);
        Ok(codegen.generate())
    }

    /// Generate JSON file in generic constraint evaluation format
    pub fn generate_gce(&self, extension_degree: u8, path: &str) -> Result<(), TestError> {
        let ir = Self::generate_ir(self)?;
        let codegen = GceCodeGenerator::new(&ir, extension_degree).map_err(|err| {
            TestError::Codegen(format!("Failed to create GCECodeGenerator: {:?}", err))
        })?;
        codegen
            .generate(path)
            .map_err(|err| TestError::Codegen(format!("Failed to generate JSON file: {:?}", err)))
    }
}
