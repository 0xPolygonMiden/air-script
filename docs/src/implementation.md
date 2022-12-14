# Language Implementation

This is a high-level overview of the implementation.

## Parser

The parser is split into 3 modules:

- a scanner built using the [Logos](https://crates.io/crates/logos) lexer-generator tool
- an LR(1) parser generated by the [LALRPOP](https://crates.io/crates/lalrpop) parser-generator framework
- an AST for representing the parsed AIR description

## IR

The IR is where semantic checking is done and where optimizations will be done in the future.

A directed acyclic graph called `AlgebraicGraph` is responsible for efficiently representing all transition constraints, identifying their type (main or auxiliary), and computing their degrees.

Currently, the checks in the IR are very minimal and cover the following simple cases.

### Identifiers

- Prevent duplicate identifier declarations.
- Prevent usage of undeclared identifiers.

### Periodic columns

- Ensure the cycle length of periodic columns is valid (greater than the minimum and a power of two).

### Boundary constraints

- Prevent multiple constraints against the same column at the same boundary.
- Ensure boundary constraint expressions contain valid identifier references by:
  - preventing periodic columns from being used by boundary constraints.
  - ensuring valid indices for public inputs.
- Identify boundary constraint types (main or auxiliary), based on the trace column to which the constraint is applied.

### Transition constraints

- Ensure transition constraints contain valid identifier references by:
  - Preventing public inputs from being used.
  - Preventing "next" indicators from being applied to anything other than trace columns.
- Identify transition constraint types (main or auxiliary) based on the constraint expression.
  - Constraints referencing the auxiliary trace or using random values are identified as constraints against the auxiliary trace.
  - All other constraints are identified as constraints against the main trace.

## Winterfell Codegen

The `codegen/winterfell` crate provides a code generator for a Rust implementation of the [Winterfell prover's](https://github.com/novifinancial/winterfell) `Air` trait from an instance of an AirScript `IR`.
