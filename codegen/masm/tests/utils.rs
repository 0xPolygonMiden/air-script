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

pub struct Data<'a, T>
where
    T: Default + std::fmt::Display,
{
    pub data: Vec<T>,
    pub address: u64,
    pub descriptor: &'a str,
}

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

/// Helper to initialize the VM's memory.
///
/// This writes the data directly to the assembly code instead of using procedures and advice
/// inputs. This makes it easy to print the code and debug directly it in the VM.
fn push_to_memory<T>(code: &mut String, memory: &mut Data<T>)
where
    T: Default + std::fmt::Display,
{
    code.push_str(&format!("    # initialize {}\n", memory.descriptor));
    code.push_str(&format!("    push.{} # memory address\n", memory.address));

    pad_to_word_len(&mut memory.data);
    for (i, data) in memory.data.chunks(4).enumerate() {
        code.push_str(&format!(
            "        push.{}.{}.{}.{} dup.4 add.{} mem_storew dropw # row {}\n",
            data[3], data[2], data[1], data[0], i, i,
        ));
    }

    code.push_str("    drop # clean address\n");
    code.push_str(&format!("    # finished {}\n\n", memory.descriptor));
}

/// Given the generated procedures as `code` and `frame_data`, returns the test code.
pub fn test_code<T>(
    mut code: String,
    memory: Vec<Data<T>>,
    trace_len: u64,
    z: QuadExtension<Felt>,
    execs: &[&str],
) -> String
where
    T: Default + std::fmt::Display,
{
    assert!(
        trace_len.is_power_of_two(),
        "trace_len must be a power of two"
    );

    // asserts there is no overlap between the memory ranges
    let mut ranges: Vec<(u64, u64)> = memory
        .iter()
        .map(|data| (data.address, data.address + data.data.len() as u64))
        .collect();
    ranges.sort();

    for check in ranges.windows(2) {
        let first = check[0];
        let second = check[1];
        assert!(first.1 <= second.0, "memory ranges overlap");
    }

    let main_memory_pos = ranges
        .binary_search_by_key(&constants::OOD_FRAME_ADDRESS, |&(address, _)| address)
        .expect("main trace memory missing");

    assert!(ranges[main_memory_pos].1 > 0, "main trace memory is empty");

    code.push_str("# END CODEGEN | START TESTCODE\n");
    code.push_str("begin\n");

    // save the trace length
    code.push_str(&format!(
        "    push.{} push.{} mem_store # trace_len\n",
        trace_len,
        constants::TRACE_LEN_ADDRESS
    ));

    // save the out-of-domain element
    let [z_0, z_1] = z.to_base_elements();
    code.push_str(&format!(
        "    push.{}.{}.0.0 push.{} mem_storew dropw # z\n\n",
        z_0.as_int(),
        z_1.as_int(),
        constants::Z_ADDRESS,
    ));

    // initialize the memory
    for mut data in memory {
        push_to_memory(&mut code, &mut data);
    }

    // call procedures to test
    for proc in execs {
        code.push_str(&format!("    exec.{}\n", proc));
    }
    code.push_str("end\n");

    code
}
