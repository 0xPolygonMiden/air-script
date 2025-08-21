//! AirScript Compiler
//!
//! This crate aggregates all components of the AirScript compiler into a single place.
//! Specifically, it re-exports functionality from the [parser](../parser/), [ir](../ir/), and
//! [winterfell code generator](../codegen/winterfell/) crates. Additionally, when compiled as an
//! executable, this crate can be used via a [CLI](#command-line-interface-cli) to transpile AIRs
//! defined in AirScript to a specified target language.
//!
//! ## Basic Usage
//!
//! An in-depth description of AirScript is available in the full AirScript [documentation](https://0xmiden.github.io/air-script/).
//!
//! The compiler has four stages, which can be imported and used independently or together.
//!
//! 1. [Parser](../parser/): scans and parses AirScript files and builds an AST
//! 2. [MIR](../mir/): produces a middle intermediate representation from the AirScript AST
//! 3. [AIR](../air/): produces an intermediate representation from an AirScript MIR
//! 4. [Code generation](../codegen/): translate an `AirIR` into a specific target language
//!    - [Winterfell Code Generator](../codegen/winterfell/): generates Rust code targeting the [Winterfell prover](https://github.com/novifinancial/winterfell).
//!
//! # Example usage
//!
//! ```rust
//! use air_script::{parse, compile, WinterfellCodeGenerator};
//! use std::sync::Arc;
//! use miden_diagnostics::{
//!     term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
//! };
//!
//! // Used for diagnostics reporting
//! let codemap = Arc::new(CodeMap::new());
//! let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
//! let diagnostics = DiagnosticsHandler::new(Default::default(), codemap.clone(), emitter);
//!
//! // Example AirScript source
//! let source = r#"
//! def Example
//!
//! trace_columns {
//!     main: [a],
//! }
//!
//! public_inputs {
//!     inputs: [16],
//! }
//!
//! boundary_constraints {
//!     enf a.first = inputs[0];
//!     enf a.last = 0;
//! }
//!
//! integrity_constraints {
//!     enf a' = a + 1;
//! }
//! "#;
//!
//! // Parse into AST
//! let ast = parse(&diagnostics, codemap, source).expect("parsing failed");
//!
//! // Compile AST into AIR
//! let air = compile(&diagnostics, ast).expect("compilation failed");
//!
//! // Generate Rust code targeting the Winterfell prover
//! let code = <WinterfellCodeGenerator as air_ir::CodeGenerator>::generate(&WinterfellCodeGenerator, &air).expect("codegen failed");
//! ```
//!
//! An example of an AIR defined in AirScript can be found in the `examples/` directory.
//!
//! # Command-Line Interface (CLI)
//!
//! There is a command-line interface available for transpiling AirScript files. Currently, the only available target is Rust code for use with the [Winterfell](https://github.com/novifinancial/winterfell) STARK prover library.
//!
//! To use the CLI, first run:
//!
//! ```bash
//! cargo build --release
//! ```
//!
//! Then, run the `airc` target with the `transpile`. For example:
//!
//! ```bash
//! ./target/release/airc transpile examples/example.air
//! ```
//!
//! When no output destination is specified, the output file will use the path and name of the input
//! file, replacing the `.air` extension with `.rs`. For the above example, `examples/example.rs`
//! will contain the generated output.
//!
//! You can use the `help` option to see other available options.
//!
//! ```bash
//! ./target/release/airc transpile --help
//! ```

pub use air_codegen_winter::CodeGenerator as WinterfellCodeGenerator;
pub use air_ir::{Air, CompileError, compile};
pub use air_parser::{parse, parse_file, transforms};
