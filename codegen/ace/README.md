# ACE Code Generator

The ACE (Arithmetic Circuit Evaluation) code generator transforms AirScript constraints into arithmetic circuits that can be efficiently evaluated by Miden VM's ACE chiplet. This enables recursive STARK proof verification within Miden assembly programs by reducing the computational overhead of constraint evaluation.

As the final stage in AirScript's compilation pipeline, this crate converts the algebraic intermediate representation (AIR) into a circuit format optimized for the ACE chiplet's execution model. The resulting circuits enable out-of-domain constraint polynomial evaluation, a critical component of recursive proof verification in zero-knowledge systems.

## Background

The ACE chiplet is designed to reduce the number of cycles required when recursively verifying STARK proofs in Miden assembly. It represents arithmetic circuits as directed acyclic graphs (DAGs) where leaf nodes are inputs and constants, intermediate nodes perform arithmetic operations, and the final root node must evaluate to zero for valid proofs.

The chiplet uses a "wiring bus" logUp argument to efficiently track node creation and consumption, enabling verifiable circuit evaluation with computational integrity guarantees. This is particularly valuable for complex computational proofs where efficient, verifiable circuit evaluation is critical.

### Mathematical Foundation

The ACE circuit computes a specific formula that combines three groups of constraint roots:
- **Integrity constraints**: Applied to every row or every frame
- **Boundary-first constraints**: Applied only to the first row  
- **Boundary-last constraints**: Applied only to the last row

These constraint groups are linearly combined with powers of a random challenge `α` and evaluated using vanishing polynomials to produce the formula:

```
z₋₂²⋅z₋₁⋅z₀⋅int + zₙ⋅z₋₂⋅bf + zₙ⋅z₀⋅bl - Q(z)⋅zₙ⋅z₀⋅z₋₂ = 0
```

Where:
- `z₀ = z - 1` (vanishing polynomial for first row)
- `z₋₂ = z - g⁻²` (vanishing polynomial for penultimate row)  
- `z₋₁ = z - g⁻¹` (vanishing polynomial for last row)
- `zₙ = zⁿ - 1` (vanishing polynomial for all rows)
- `Q(z)` is the reconstructed quotient polynomial
- `n` is the trace length

## Architecture

### Circuit Structure

The ACE circuit uses a DAG representation with three types of nodes:

```rust
pub enum Node {
    Input(usize),     // Variable at which circuit is evaluated
    Constant(usize),  // Fixed value stored in circuit description  
    Operation(usize), // Result of arithmetic operation on two nodes
}
```

Operations are limited to three arithmetic operations supported by the chiplet:
- `Sub` (subtraction)
- `Mul` (multiplication) 
- `Add` (addition)

### Input Layout

The ACE chiplet expects inputs in a specific memory layout defined by `AirLayout`:

1. **Public Inputs**: Values from AirScript `public_inputs` declarations as well as variable-sized ones used for bus boundary constraints
2. **Random Values**: Auxiliary randomness (α, β challenges)
3. **Trace Segments**: 
   - Main segment evaluations (current and next row)
   - Auxiliary segment evaluations (current and next row)
4. **Quotient Evaluations**: 8 quotient polynomial parts `[Q₀(z), ..., Q₇(z)]`
5. **STARK Variables**: `[α, z, zⁿ, g⁻¹, zᵐᵃˣ, g⁻²]`

Each region is aligned to specific boundaries (word, double-word, quad-word) to match the recursive verifier's memory access patterns.

### Circuit Encoding

The `EncodedAceCircuit` serializes circuits into the format expected by the ACE chiplet:

- **Variables**: Input and constant nodes stored as extension field elements
- **Instructions**: Arithmetic operations encoded as field elements with packed opcodes and node indices
- **Padding**: Unused instructions filled with squaring operations (which preserve zero evaluation)

## API

### Main Entry Point

```rust
pub fn build_ace_circuit(air: &Air) -> anyhow::Result<(AceNode, AceCircuit)>
```

Converts an AIR representation to an ACE circuit, returning the root node and complete circuit.

### Key Types

- **`AceCircuit`**: Complete circuit representation with layout and operations
- **`AceNode`**: Type alias for `Node` representing circuit nodes
- **`EncodedAceCircuit`**: Serialized circuit format for chiplet consumption
- **`AceVars`**: Input variables structure matching chiplet requirements
- **`AirLayout`**: Memory layout specification for input organization

## Usage Example

```rust
use air_codegen_ace::{build_ace_circuit, AceVars};
use air_ir::{Air, compile};

// Parse AirScript source
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

## Testing & Debugging

### Test Utilities

```rust
// Generate circuit from AirScript source
let (air, circuit, root) = generate_circuit(source);

// Create random inputs with valid quotient
let ace_vars = AceVars::random_with_valid_quotient(&air, log_trace_len);

// Evaluate and verify
let result = circuit.eval(root, &ace_vars.to_memory_vec(&circuit.layout));
```

### Visualization Support

Circuits can be exported to Graphviz DOT format for debugging:

```rust
let dot_graph = circuit.to_dot()?;
// View at https://dreampuf.github.io/GraphvizOnline
// Or: dot -Tsvg circuit.dot > circuit.svg
```

### Test Coverage

The crate includes comprehensive tests:
- **Randomized Testing**: Validates all test circuits with random inputs
- **Regression Testing**: Generates DOT files for visual regression testing
- **Quotient Validation**: Ensures proper quotient polynomial reconstruction

## Dependencies

- **`miden-core`**: Core Miden VM types and cryptographic primitives
- **`winter-math`**: Finite field arithmetic and STARK-specific mathematics  
- **`air-ir`**: AIR intermediate representation from the compilation pipeline
- **`air-mir`**: MIR types, particularly `QuadFelt` for extension field elements

## References

- [ACE Chiplet Documentation](https://0xmiden.github.io/miden-vm/design/chiplets/ace.html)