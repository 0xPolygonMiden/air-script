# AirScript Compiler

This crate aggregates all components of the AirScript compiler into a single place. Specifically, it re-exports functionality from the [parser](../parser/), [ir](../ir/), and [winterfell code generator](../codegen/winterfell/) crates. Additionally, when compiled as an executable, this crate can be used via a [CLI](#command-line-interface-cli) to transpile AIRs defined in AirScript to a specified target language.

## Basic Usage

An in-depth description of AirScript is available in the full AirScript [documentation](https://0xpolygonmiden.github.io/air-script/).

The compiler has three stages, which can be imported and used independently or together.

1. [Parser](../parser/): scans and parses AirScript files and builds an AST
2. [IR](../ir/): produces an intermediate representation from an AirScript AST
3. [Code generation](../codegen/): translate an `AirIR` into a specific target language
   - [Winterfell Code Generator](../codegen/winterfell/): generates Rust code targeting the [Winterfell prover](https://github.com/novifinancial/winterfell).

Example usage:

```Rust
use air_script::{parse, AirIR, CodeGenerator};

// parse the source string to a Result containing the AST or an Error
let ast = parse(source.as_str()).expect("Parsing failed");

// process the AST to get a Result containing the AirIR or an Error
let ir = AirIR::new(&ast).expect("AIR is invalid");

// generate Rust code targeting the Winterfell prover
let rust_code = CodeGenerator::new(&ir);
```

An example of an AIR defined in AirScript can be found in the `examples/` directory.

To run the full transpilation pipeline, the CLI can be used for convenience.

## Command-Line Interface (CLI)

There is a command-line interface available for transpiling AirScript files. Currently, the only available target is Rust code for use with the [Winterfell](https://github.com/novifinancial/winterfell) STARK prover library.

To use the CLI, first run:

```
cargo build --release
```

Then, run the `airc` target with the `transpile`. For example:

```
./target/release/airc transpile examples/example.air
```

When no output destination is specified, the output file will use the path and name of the input file, replacing the `.air` extension with `.rs`. For the above example, `examples/example.rs` will contain the generated output.

You can use the `help` option to see other available options.

```
./target/release/airc transpile --help
```
