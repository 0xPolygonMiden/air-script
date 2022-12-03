# Introduction

Polygon Miden's AirScript is designed to make it simple to describe AIR constraints and generate efficient and accurate code in the required target language. The code for AirScript can be found [here](https://github.com/0xPolygonMiden/air-script/).

## Current Version

Currently, AirScript is on version 0.1, which supports a simple syntax for describing air constraints and generation of Rust code targeting the [Winterfell prover](https://github.com/novifinancial/winterfell).

The simplified version of the language is based on [this discussion](https://github.com/maticnetwork/miden/discussions/254), but only includes the following:

- declaring trace columns for main and auxiliary traces as a vector (e.g. `main: [a, b, c]` but not `main[a, b, c[2]]`)
- declaring public inputs where each public input is a named vector (e.g. `stack_inputs: [16]`)
- declaring periodic columns
- enforcing boundary constraints for main and auxiliary traces using trace columns, public inputs, and inline scalar constants
- enforcing transition constraints for main and auxiliary traces using trace columns, periodic columns, and inline scalar constants

The language will be specified in detail in the rest of this book.

### CLI

There is a command-line interface available for transpiling AirScript files to Rust. There are also several example `.air` files written in AirScript which can be found in the `examples/` directory.

To use the CLI, first run:

```
cargo build --release
```

Then, run the `airc` target with the `transpile` option and specify your input file with `-i`. For example:

```
./target/release/airc transpile -i examples/example.air
```

You can use the `help` option to see other available options.

```
./target/release/airc transpile --help
```

## Future Work

The following changes are some of the improvements under consideration for future releases.

- more advanced language functionality for better ergonomics and modularity, such as:
  - modules and imports
  - variable declarations (e.g. `let x = k1 * c[1]'`)
  - named constants (e.g. in a `constants` section), including:
    - vector constants
    - matrix constants
  - python-style list comprehension (and other "convenience" syntax)
  - support for functions
  - support for evaluators
  - support for selectors
- optimizations, such as:
  - removing unnecessary nodes from the `AlgebraicGraph` of transition constraints
  - combining transition constraints with mutually exclusive selectors to reduce the total number of constraints
- additional language targets for simplifying verifier implementations:
  - Solidity
  - Miden Assembly
- formal verification
