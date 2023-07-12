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
use air_script::{Air, parse, passes, Pass, transforms, WinterfellCodeGenerator};
use miden_diagnostics::{
    term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
};

// Used for diagnostics reporting
let codemap = Arc::new(CodeMap::new());
let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
let diagnostics = DiagnosticsHandler::new(Default::default(), codemap.clone(), emitter);

// Parse into AST
let ast = parse(&diagnostics, codemap, source.as_str()).expect("parsing failed");
// Lower to IR
let air = {
   let mut pipeline = transforms::ConstantPropagation::new(&diagnostics)
      .chain(transforms::Inlining::new(&diagnostics))
      .chain(passes::AstToAir::new(&diagnostics));
   pipeline.run(ast).expect("lowering failed")
};

// Generate Rust code targeting the Winterfell prover
let code = WinterfellCodeGenerator::new(&ir).generate().expect("codegen failed");
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
