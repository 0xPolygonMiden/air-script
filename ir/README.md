# Intermediate Representation (IR)

This crate contains the intermediate representation for AirScript, `AirIR`.

The purpose of the `AirIR` is to provide a simple and accurate representation of an AIR that allows for optimization and translation to constraint evaluator code in a variety of target languages.

## Generating the AirIR

Generate an `AirIR` from an AirScript AST (the output of the AirScript parser) using the `from_source` method. The `from_source` method will return a new `AirIR` or an `Error` of type `SemanticError` if it encounters any errors while processing the AST.

The `from_source` method will first iterate through the source sections that contain declarations to build a symbol table with trace columns, public inputs, and periodic columns. It will return a `SemanticError` if it encounters a duplicate, incorrect, or missing declaration. Once the symbol table is built, the constraints in the `boundary_constraints` and `transition_constraints` sections of the AST are processed. Finally, `from_source` returns a Result containing the `AirIR` or a `SemanticError`.

Example usage:

```Rust
// parse the source string to a Result containing the AST or an Error
let ast = parse(source.as_str()).expect("Parsing failed");

// process the AST to get a Result containing the AirIR or an Error
let ir = AirIR::from_source(&ast)
```

## AirIR

Although generation of an `AirIR` uses a symbol table while processing the source AST, the internal representation only consists of the following:

- **Name** of the AIR definition represented by the `AirIR`.
- **Public inputs**, represented by a vector that maps an identifier to a size for each public input that was declared. (Currently, public inputs can only be declared as fixed-size arrays.)
- **Periodic columns**, represented by an ordered vector that contains each periodic column's repeating pattern (as a vector).
- **Boundary constraints**, stored as mappings from trace column indices to expressions, with a separate mapping for each boundary (first and last) of each trace segment (main and auxiliary).
- **Transition constraints**, represented by the combination of:
  - a directed acyclic graph (DAG) without duplicate nodes.
  - a vector of `NodeIndex` for each of the main and auxiliary trace segments, which contains the node index in the graph where each of the respective trace segment's constraints starts.
