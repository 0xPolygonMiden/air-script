# Introduction

Polygon Miden's AirScript is designed to make it simple to describe AIR constraints and generate efficient and accurate code in the required target language. The code for AirScript can be found [here](https://github.com/0xPolygonMiden/air-script/).

## Current Version

Currently, AirScript is on version 0.2, which supports a simple syntax for describing air constraints and generation of Rust code targeting the [Winterfell prover](https://github.com/novifinancial/winterfell).

The language is based on [this discussion](https://github.com/maticnetwork/miden/discussions/254), and includes features to write ~90% of the constraints required for Miden albeit with some limitations.

AirScript includes the following features:

- **Trace Columns**: Users can declare trace columns for main and auxiliary traces as individual columns or groups of columns (e.g. `main: [a, b, c[3], d]` where `a`, `b`, and `d` are single columns and `c` refers to a group of 3 columns)

- **Public Inputs**: Users can declare public inputs where each public input is a named vector (e.g. `stack_inputs: [16]`)

- **Periodic Columns**: Users can declare periodic columns (e.g. `k0: [1, 0, 0, 0]`)

- **Random Values**: Users can define random values provided by the verifier (e.g. `alphas: [x, y[14], z]` or `rand: [16]`)

- **Boundary Constraints**: Users can enfore boundary constraints on main and auxiliary trace columns using public inputs, random values, constants and variables.

- **Integrity Constraints**: Users can enforce integrity constraints on main and auxiliary trace columns using trace columns, periodic columns, random values, constants and variables.

- **Constants**: Users can declare module level constants. Constants can be scalars, vectors or matrices.
  (e.g. `const A = 123`, `const B = [1, 2, 3]`, `const C = [[1, 2, 3], [4, 5, 6]]`)

- **Variables**: Local variables can be declared for use in defining boundary and integrity constraints. Variables can be scalars, vectors or matrices built from expressions (e.g. `let x = k * c[1]'`, `let y = [k * c[1], l * c[2], m * c[3]]` or `let z = [[k * c[1], l * c[2]], [m * c[3], n * c[4]]]`)

The language also includes some convenience syntax like list comprehension and list folding to make writing constraints easier.
(e.g. `let x = [k * c for (k, c) in (k, c[1..4])]`, `let y = sum([k * c for (k, c) in (k, c[1..4])])`)

The language will be specified in detail in the rest of this book.

### CLI

There is a command-line interface available for transpiling AirScript files to Rust. There are also several example `.air` files written in AirScript which can be found in the `examples/` directory.

To use the CLI, first run:

```
cargo build --release
```

Then, run the `airc` target with the `transpile` option. For example:

```
./target/release/airc transpile examples/example.air
```

You can use the `help` option to see other available options.

```
./target/release/airc transpile --help
```

## Future Work

The following changes are some of the improvements under consideration for future releases.

- more advanced language functionality for better ergonomics and modularity, such as:
  - python-style list comprehension (and other "convenience" syntax)
  - modules and imports
  - support for functions
  - support for evaluators
  - support for selectors
- optimizations, such as:
  - constant folding
  - removing unnecessary nodes from the `AlgebraicGraph` of boundary and integrity constraints
  - combining integrity constraints with mutually exclusive selectors to reduce the total number of constraints
- additional language targets for simplifying verifier implementations:
  - Solidity
  - Miden Assembly
- formal verification
