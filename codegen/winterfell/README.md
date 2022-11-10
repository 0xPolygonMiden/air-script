# Winterfell Code Generator

This crate contains a code generator targeting the [Winterfell prover](https://github.com/novifinancial/winterfell) Rust library.

The purpose of this code generator is to convert a provided `AirIR` representation of an AIR into a custom Rust struct that implements Winterfell's `Air` trait. The generated code can be used instead of writing a custom Winterfell `Air` implementation directly in Rust.

## Generating the Winterfell Rust Code

Generate Rust code from an `AirIR` (AirScript's intermediate representation) by instantiating a new `CodeGenerator` with an AirScript AST (the output of the AirScript parser) and then calling `generate`. The `generate` method will return the Rust code implementation as a `String`.

Instantiating the `CodeGenerator` will add the required Winterfell imports, create a custom `struct` using the name defined for the AIR, then implement the Winterfell `Air` trait for the custom `struct`.

Example usage:

```Rust
// parse the source string to a Result containing the AST or an Error
let ast = parse(source.as_str()).expect("Parsing failed");

// process the AST to get a Result containing the AirIR or an Error
let ir = AirIR::from_source(&ast).expect("AIR is invalid");

// generate Rust code targeting the Winterfell prover
let rust_code = CodeGenerator::new(&ir);
```

## Generated Winterfell Rust Code

The following code is generated for the Winterfell `Air` trait implementation:

- declaration and implementation of a `PublicInputs` struct.
- custom struct declaration and implementation, using the defined name of the AIR from the original AirScript file
- implementation of Winterfell `Air` trait:
  - constraint-related declarations as part of the `AirContext` creation in the `new` method:
    - the number of boundary constraints for the main trace
    - the number of boundary constraints for the auxiliary trace
    - the order and degrees of the transition constraints for the main trace
    - the order and degrees of the transition constraints for the auxiliary trace
  - getters for:
    - periodic column values (`get_periodic_column_values`)
    - main trace boundary constraints (`get_assertions`)
    - auxiliary trace boundary constraints (`get_aux_assertions`)
  - transition constraint evaluation code for:
    - main trace transition constraints (`evaluate_transition`)
    - auxiliary trace transition constraints (`evaluate_aux_transition`)
