use std::sync::Arc;

use air_codegen_ace::{build_ace_circuit, AceCircuit, AceNode};
use air_ir::{compile, Air};
use miden_diagnostics::{
    term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
};
use winter_math::FieldElement;

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

/// Loads the MidenVM AIR example
pub fn load_miden_vm_air() -> std::io::Result<String> {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = format!("{}/miden_vm.air", crate_dir);
    let content = std::fs::read_to_string(path)?;

    Ok(content)
}

#[test]
fn test_miden_vm_updated_air_randomized() {
    let air_string = load_miden_vm_air().expect("unable to read MidenVM AIR");

    let (_air, circuit, root_node) = generate_circuit(&air_string);

    // Provide dummy variable assignments since we are not generating valid ACE vars here
    let dummy_inputs = vec![Default::default(); circuit.layout.num_inputs];
    let eval = circuit.eval(root_node, &dummy_inputs);

    assert_eq!(eval, <_ as FieldElement>::ZERO);
}
