use std::sync::Arc;

use air_ir::{compile, Air};
use miden_diagnostics::{
    term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
};
use winter_math::FieldElement;
use winter_math::fields::f64::BaseElement as Felt;
use miden_core::crypto::hash::Rpo256;

use air_codegen_ace::{build_ace_circuit, AceCircuit, AceNode};

/// Local copy of generate_circuit, since ace::tests version is not public
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
/// Loads the MidenVM AIR example next to this test
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

    let encoded_circuit = circuit.to_ace();

    let circuit_description: Vec<Felt> = CIRCUIT_DESCRIPTION.into_iter().map(Felt::new).collect();
    let circuit_hash_expected = Rpo256::hash_elements(&circuit_description);
    
    assert_eq!(eval, <_ as FieldElement>::ZERO);
    assert_eq!(encoded_circuit.circuit_hash(), CIRCUIT_HASH.into());
    assert_eq!(encoded_circuit.circuit_hash(), circuit_hash_expected);
}

const CIRCUIT_HASH: [Felt; 4] = [
    Felt::new(9139186206676821480),
    Felt::new(12763675724578443945),
    Felt::new(7621207635344139731),
    Felt::new(1122100627503939866),
];

const CIRCUIT_DESCRIPTION: [u64; 112] = [
    1,
    0,
    0,
    0,
    2305843126251553075,
    114890375379,
    2305843283017859381,
    2305843266911732021,
    1152921616275996777,
    2305843265837990197,
    1152921614128513127,
    2305843319525081397,
    1152921611981029477,
    2305843318451339573,
    1152921609833545827,
    2305843317377597749,
    1152921607686062177,
    2305843316303855925,
    1152921605538578527,
    1152921604464836831,
    1152921614128513128,
    1152921602317353060,
    1152921601243611234,
    1152921600169869408,
    1152921599096127582,
    1152921598022385920,
    2305843101555490908,
    1152921604464836735,
    1152921614128513129,
    1152921593727418468,
    1152921592653676642,
    1152921591579934816,
    1152921590506192990,
    1152921776263528702,
    270582939757,
    1152921587284967502,
    1152921586211225679,
    1152921611981029479,
    1152921584063742050,
    1152921582990000224,
    1152921581916258398,
    1152921580842516556,
    2305843084375621707,
    1152921609833545829,
    1152921577621291104,
    1152921576547549278,
    315680096365,
    1152921574400065828,
    314606354541,
    1152921572252581952,
    1152921571178840130,
    2305843074711945285,
    1152921607686062179,
    1152921567957614686,
    1152921566883872830,
    2305843070416977980,
    1152921605538578529,
    1152921563662647358,
    2305843067195752504,
    1152921571178840159,
    2305843065048268853,
    2305843063974527060,
    53687091285,
    333933707485,
    117037858930,
    118111600754,
    120259084402,
    117037858929,
    1152921557220196467,
    2305843055384592490,
    1152921553998970927,
    1152921548630261805,
    1152921547556519982,
    1152921546482778154,
    1152921628087156851,
    1152921544335294771,
    1152921544335294579,
    1152921542187811039,
    2305843045720916004,
    1152921542187810931,
    1152921538966585392,
    2305843042499690529,
    1152921551851487278,
    1152921535745359902,
    2305843039278465062,
    1152921538966585459,
    1152921532524134623,
    1152921551851487279,
    1152921530376650777,
    2305843033909755931,
    1152921625939673300,
    2305843031762272469,
    1152921526081683569,
    2305843029614788822,
    1152921523934199921,
    2305843027467305175,
    1152921521786716273,
    2305843025319821528,
    1152921519639232625,
    2305843023172337881,
    1152921517491748977,
    2305843021024854234,
    1152921515344265329,
    2305843018877370587,
    1152921548630261804,
    1152921512123039752,
    6442450966,
    1152921509975556101,
    1152921508901814276,
    1152921507828072451,
    1152921506754330626,
    1152921505680588801,
];
