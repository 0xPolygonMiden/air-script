# AirScript

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/0xMiden/air-script/blob/main/LICENSE)
[![test](https://github.com/0xMiden/air-script/actions/workflows/test.yml/badge.svg)](https://github.com/0xMiden/air-script/actions/workflows/test.yml)
[![build](https://github.com/0xMiden/air-script/actions/workflows/build.yml/badge.svg)](https://github.com/0xMiden/air-script/actions/workflows/build.yml)
[![RUST_VERSION](https://img.shields.io/badge/rustc-1.89+-lightgray.svg)](https://www.rust-lang.org/tools/install)
[![Crates.io](https://img.shields.io/crates/v/air-script)](https://crates.io/crates/air-script)

A domain-specific language for expressing AIR constraints for STARKs, especially for STARK-based virtual machines like [Miden VM](https://github.com/0xMiden/miden-vm).

An in-depth description of AirScript is available in the full AirScript [documentation](https://0xMiden.github.io/air-script/).

**WARNING**: This project is in an alpha stage. It has not been audited and may contain bugs and security flaws. This implementation is NOT ready for production use.

## Overview

AirScript is a domain-specific language for writing AIR constraints for the STARK proving system. The primary goal of AirScript is to enable writing and auditing AIR constraints without the need to learn a specific programming language (e.g., Rust). The secondary goal is to perform automated optimizations of constraints and to output constraint evaluator code in multiple target languages (e.g., Rust, Miden assembly, Solidity etc.).

## Project Structure

The project is organized into several crates as follows:
| Crate | Description |
| ---------------------- | ----------- |
| [Parser](parser) | Contains the parser for AirScript. The parser is used to parse the constraints written in AirScript into an AST. |
| [MIR](mir) | Contains the middle intermediate representation (`MIR`). The purpose of the `MIR` is to provide a representation of an AirScript program that allows for optimization and translation to `AirIR` containing the `AlgebraicGraph`. |
| [AIR](air) | Contains the IR for AirScript, `AirIR`. `AirIR` is initialized with an AirScript MIR, which it converts to an internal representation that can be optimized and used to generate code in multiple target languages. |
| [Winterfell code generator](codegen/winterfell/) | Contains a code generator targeting the [Winterfell prover](https://github.com/novifinancial/winterfell) Rust library. The Winterfell code generator converts a provided AirScript `AirIR` into Rust code that represents the AIR as a new custom struct that implements Winterfell's `Air` trait. |
| [ACE code generator](codegen/ace/) | Contains a code generator targeting Miden VM's ACE (Arithmetic Circuit Evaluation) chiplet. Converts AirScript constraints into arithmetic circuits optimized for recursive STARK proof verification within Miden assembly programs. |
| [AirScript](air-script) | Aggregates all components of the AirScript compiler into a single place and provides a CLI as an executable to transpile AIRs defined in AirScript to the specified target language. Also contains integration tests for AirScript. |

## Documentation and Examples

AirScript documentation uses mdBook and is located in the `docs/` directory. Examples are stored in `docs/examples/` and are included in the documentation using mdBook's include syntax.

### Adding New Examples

To add a new example to the documentation:

1. **Create the example file**: Add your `.air` file to `docs/examples/`
2. **Test compilation**: Ensure the example compiles using the CLI:
   ```bash
   cargo build --release
   target/release/airc transpile docs/examples/your_example.air -o /tmp/test.rs
   ```
3. **Include in documentation**: Add the example to the relevant markdown file in `docs/src/` using:
   
      ```air
      {{#include ../../examples/your_example.air}}
      ```
      
### Testing Documentation Examples

The project includes an integration test that ensures all documentation examples compile successfully:

```bash
cargo test -p air-script --test docs_sync
```

This test automatically:
- Builds the AirScript CLI tool
- Finds all `.air` files in `docs/examples/`
- Transpiles each example to verify compilation
- Cleans up generated files

The `docs_sync` test runs as part of the CI pipeline to ensure documentation examples remain valid and up-to-date.

### Documentation Build Test

The project also includes a documentation build test that runs as part of CI:

```bash
make test-docs
```

This test ensures that the documentation builds correctly and validates that `docs/build.rs` runs properly when the `DOCS_TEST` environment variable is set.

## Contributing to AirScript

AirScript is an open project and we welcome everyone to contribute! If you are interested in contributing to AirScript, please have a look at our [Contribution guidelines](https://github.com/0xMiden/air-script/blob/main/CONTRIBUTING.md). If you want to work on a specific issue, please add a comment on the GitHub issue indicating you are interested before submitting a PR. This will help avoid duplicated effort. If you have thoughts on how to improve AirScript, we'd love to know them. So, please don't hesitate to open issues.

## References

1. [Logos](https://github.com/maciejhirsz/logos/): Library for generating fast lexers in Rust.
1. [LALRPOP](https://github.com/lalrpop/lalrpop/): LR(1) Rust parser generator framework.
1. [Codegen](https://github.com/carllerche/codegen): Library for generating Rust code.
1. [mdBook](https://github.com/rust-lang/mdBook): Utility for creating online documentation books.

## License

This project is [MIT licensed](./LICENSE).
