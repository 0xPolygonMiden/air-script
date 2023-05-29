use ir::AirIR;
use std::fs;

#[derive(Debug)]
pub enum TestError {
    IO(String),
    Parse(String),
    IR(String),
}

pub enum Target {
    Winterfell,
    Masm,
}

pub struct Test {
    input_path: String,
}

impl Test {
    pub fn new(input_path: String) -> Self {
        Test { input_path }
    }

    pub fn transpile(&self, target: Target) -> Result<String, TestError> {
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

        let ir = AirIR::new(parsed).map_err(|_| {
            TestError::IR(format!(
                "Failed to convert the input air file at {} to IR representation",
                &self.input_path
            ))
        })?;

        let code = match target {
            Target::Winterfell => codegen_winter::CodeGenerator::new(&ir).generate(),
            Target::Masm => {
                codegen_masm::CodeGenerator::new(&ir, codegen_masm::CodegenConfig::default())
                    .generate()
                    .expect("code generation failed")
            }
        };

        // generate Rust code targeting Winterfell
        Ok(code)
    }
}

/// Parses the provided source and returns the AST.
fn parse(source: &str) -> Result<parser::ast::Source, parser::ParseError> {
    use miden_diagnostics::{term::termcolor::ColorChoice, *};
    use parser::{ast, ParseError, Parser};
    use std::sync::Arc;

    let codemap = Arc::new(CodeMap::new());
    let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
    let config = DiagnosticsConfig {
        verbosity: Verbosity::Warning,
        warnings_as_errors: true,
        no_warn: false,
        display: Default::default(),
    };
    let diagnostics = DiagnosticsHandler::new(config, codemap.clone(), emitter);
    let parser = Parser::new((), codemap);
    match parser.parse_string::<ast::Source, _, _>(&diagnostics, source) {
        Ok(ast) => Ok(ast),
        Err(ParseError::Lexer(err)) => {
            diagnostics.emit(err);
            Err(ParseError::Failed)
        }
        Err(err) => Err(err),
    }
}
