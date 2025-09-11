# ACE Code Generator

This crate contains a code generator targeting Miden VM's ACE (Arithmetic Circuit Evaluation) chiplet.

The purpose of this code generator is to convert a provided `Air` representation into arithmetic circuits that can be efficiently evaluated by the ACE chiplet for recursive STARK proof verification within Miden assembly programs.

## Generating the ACE Circuit

Generate an ACE circuit from an `Air` (AirScript's intermediate representation) by calling `build_ace_circuit`. The function will return the root node and complete circuit.

The circuit builder processes constraints in three groups (integrity, boundary-first, and boundary-last constraints), combines them using powers of a random challenge `α`, and creates a circuit that evaluates the formula:

```
z₋₂²⋅z₋₁⋅z₀⋅int + zₙ⋅z₋₂⋅bf + zₙ⋅z₀⋅bl - Q(z)⋅zₙ⋅z₀⋅z₋₂ = 0
```

Example usage:

```rust
use air_codegen_ace::{build_ace_circuit, AceVars};
use air_ir::{Air, compile};

// Parse AirScript source and compile to AIR
let air = compile(&diagnostics, parsed_program)?;

// Build ACE circuit
let (root_node, circuit) = build_ace_circuit(&air)?;

// Collect inputs to circuit
// let air_inputs = ...

// Prepare inputs for evaluation
let ace_vars = AceVars::from_air_inputs(air_inputs, &air);
let memory_inputs = ace_vars.to_memory_vec(&circuit.layout);

// Evaluate circuit (should be zero for valid constraints)
let result = circuit.eval(root_node, &memory_inputs);
assert_eq!(result, QuadFelt::ZERO);

// Encode for chiplet consumption
let encoded = circuit.to_ace();
```

## ACE Circuit

The ACE circuit represents constraints as a directed acyclic graph (DAG) with three types of nodes:
- `Input(usize)`: Variables at which the circuit is evaluated
- `Constant(usize)`: Fixed values stored in the circuit description
- `Operation(usize)`: Results of arithmetic operations (subtraction, multiplication, addition)

The circuit includes:
- **Layout** (`AirLayout`): Defines the memory layout of inputs expected by the ACE chiplet
- **Encoded format** (`EncodedAceCircuit`): Serialized representation for chiplet consumption
- **Evaluation**: Circuits can be evaluated at given inputs and visualized as DOT graphs for debugging

## References

- [ACE Chiplet Documentation](https://0xmiden.github.io/miden-vm/design/chiplets/ace.html)
