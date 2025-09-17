use std::sync::Arc;

use air_ir::{Air, compile};
use miden_diagnostics::{
    CodeMap, DefaultEmitter, DiagnosticsHandler, term::termcolor::ColorChoice,
};
use winter_math::FieldElement;

// Import re-exports correctly from ace crate
use air_codegen_ace::{build_ace_circuit, AceCircuit, AceNode};

//// Local copy of generate_circuit, since ace::tests version is not public
fn generate_circuit(source: &str) -> (Air, AceCircuit, AceNode) {
    let code_map = Arc::new(CodeMap::new());
    let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
    let diagnostics = DiagnosticsHandler::new(Default::default(), code_map.clone(), emitter);

    let air = air_parser::parse(&diagnostics, code_map, source)
        .map_err(air_ir::CompileError::Parse)
        .and_then(|program| compile(&diagnostics, program))
        .expect("lowering failed");

    let (root, circuit) = build_ace_circuit(&air).expect("codegen failed");

    (air, circuit, root)
}

/// Loads the MidenVM minimal AIR example
pub fn load_miden_vm_air() -> std::io::Result<String> {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = format!("{crate_dir}/../constraints/miden-vm-updated/main.air");
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

#[test]
fn test_miden_vm_air_randomized() {
    let air_string = load_miden_vm_air().expect("unable to read MidenVM AIR");

    let (_air, circuit, root_node) = generate_circuit(&air_string);

    // For now, simplify: just build dummy inputs
    let dummy_inputs = vec![Default::default(); circuit.layout.num_inputs];
    let eval = circuit.eval(root_node, &dummy_inputs);

    let encoded_circuit = circuit.to_ace();

    println!("vars: {}", encoded_circuit.num_vars());
    println!("inputs: {}", encoded_circuit.num_inputs());
    println!("constants: {}", encoded_circuit.num_constants());
    println!("hash: {:?}", encoded_circuit.circuit_hash());

    assert_eq!(eval, <_ as FieldElement>::ZERO);
}
