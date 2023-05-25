use air_codegen_masm::constants;
use miden_diagnostics::{
    term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
};
use parser::{ast::Source, Parser};
use processor::{
    math::{Felt, StarkField},
    QuadExtension,
};
use std::sync::Arc;

pub fn parse(source: &str) -> Source {
    let codemap = Arc::new(CodeMap::new());
    let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
    let diagnostics = DiagnosticsHandler::new(Default::default(), codemap.clone(), emitter);
    let parser = Parser::new((), codemap);

    parser
        .parse_string::<Source, _, _>(&diagnostics, source)
        .expect("parsing failed")
}

pub fn to_stack_order(values: &[QuadExtension<Felt>]) -> Vec<u64> {
    values
        .iter()
        .flat_map(|el| {
            let [el0, el1] = el.to_base_elements();
            [el1.as_int(), el0.as_int()]
        })
        .collect()
}

/// If necessary pad with zeros the data vector to the word size
fn pad_to_word_len<T>(data: &mut Vec<T>)
where
    T: Default,
{
    let last = data.len() % 4;
    if last != 0 {
        let extra = 4 - last;
        data.resize_with(data.len() + extra, || T::default());
    }
}

fn push_data_to_code<T>(code: &mut String, addr: u64, data: Vec<T>)
where
    T: std::fmt::Display,
{
    code.push_str(&format!("    push.{} # push address\n", addr,));

    for (i, data) in data.chunks(4).enumerate() {
        code.push_str(&format!(
            "        push.{}.{}.{}.{} dup.4 add.{} mem_storew dropw # load data row {}\n",
            data[3], data[2], data[1], data[0], i, i,
        ));
    }

    code.push_str("    drop # clean address\n");
}

/// Given the generated procedures as `code` and `frame_data`, returns the test code.
pub fn test_code<T>(
    mut code: String,
    mut main_frame_data: Vec<T>,
    mut aux_frame_data: Vec<T>,
    trace_len: u64,
    z: QuadExtension<Felt>,
) -> String
where
    T: Default + std::fmt::Display,
{
    assert!(
        trace_len.is_power_of_two(),
        "trace_len must be a power of two"
    );
    assert!(
        main_frame_data.len() != 0,
        "main frame data must be non-empty"
    );

    pad_to_word_len(&mut main_frame_data);
    pad_to_word_len(&mut aux_frame_data);

    let main_frame_end: u64 =
        constants::OOD_FRAME_ADDRESS + u64::try_from(main_frame_data.len()).unwrap();
    let has_space = main_frame_end <= constants::OOD_AUX_FRAME_ADDRESS;
    assert!(has_space, "main frame data would overwrite aux frame");

    code.push_str("# END CODEGEN | START TESTCODE\n");
    code.push_str("begin\n");
    code.push_str(&format!(
        "    push.{} push.{} mem_store # trace_len\n",
        trace_len,
        constants::TRACE_LEN_ADDRESS
    ));

    let [z_0, z_1] = z.to_base_elements();
    code.push_str(&format!(
        "    push.{}.{}.0.0 push.{} mem_storew dropw # z\n",
        z_0.as_int(),
        z_1.as_int(),
        constants::Z_ADDRESS,
    ));
    code.push_str("    # main trace data \n");
    push_data_to_code(&mut code, constants::OOD_FRAME_ADDRESS, main_frame_data);
    if aux_frame_data.len() > 0 {
        code.push_str("    # aux trace data \n");
        push_data_to_code(&mut code, constants::OOD_AUX_FRAME_ADDRESS, aux_frame_data);
    }
    code.push_str("    exec.cache_periodic_polys\n");
    code.push_str("    exec.compute_evaluate_transitions\n");
    code.push_str("end\n");

    code
}
