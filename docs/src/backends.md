# Backends
AirScript currently comes bundled with three backends:

- [Winterfell backend](https://github.com/0xMiden/air-script/tree/main/codegen/winterfell) which outputs `Air` trait implementation for the [Winterfell prover](https://github.com/facebook/winterfell) (Rust).
- [Plonky3 backend](https://github.com/0xMiden/air-script/tree/main/codegen/plonky3) which outputs `Air` trait implementation for the [Plonky3 prover](https://github.com/Plonky3/Plonky3) (Rust).
- [ACE backend](https://github.com/0xMiden/air-script/tree/main/codegen/ace) which outputs arithmetic circuits for Miden VM's ACE (Arithmetic Circuit Evaluation) chiplet for recursive STARK proof verification.

These backends can be used programmatically as crates.

The Winterfell and Plonky3 backends can also be used via AirScript CLI by specifying `--target` flag. For example, the following will output Winterfell and Plonky3 `Air` trait implementation for AIR constraints described in `example.air` file:
```bash
# Make sure to run from the project root directory
./target/release/airc transpile examples/example.air --target winterfell
./target/release/airc transpile examples/example.air --target plonky3
```
In both cases we assumed that the CLI has been compiled as described [here](./introduction.md#cli).
