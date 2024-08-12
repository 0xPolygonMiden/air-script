use air_codegen_masm::constants;
use miden_assembly::Assembler;
use miden_processor::{
    math::{Felt, FieldElement, StarkField},
    AdviceInputs, Kernel, MemAdviceProvider, Process, QuadExtension, StackInputs,
};

mod utils;
use utils::{codegen, test_code, to_stack_order, Data};

static SIMPLE_AIR: &str = "
def Simple

trace_columns {
    main: [a],
}

public_inputs {
    stack_inputs: [16],
}

boundary_constraints {
    enf a.first = 0;
}

integrity_constraints {
    enf a + a = 0;
}";

#[test]
fn test_exemption_points() {
    let one = QuadExtension::new(Felt::new(1), Felt::ZERO);
    let z = one;
    let a = QuadExtension::new(Felt::new(3), Felt::ZERO);
    let a_prime = a;

    for power in 3..32 {
        let trace_len = 2u64.pow(power);

        let code = codegen(SIMPLE_AIR);
        let code = test_code(
            code,
            vec![Data {
                data: to_stack_order(&[a, a_prime]),
                address: constants::OOD_FRAME_ADDRESS,
                descriptor: "main_trace",
            }],
            trace_len,
            z,
            &["get_exemptions_points"],
        );
        let program = Assembler::default().compile(code).unwrap();

        let mut process: Process<MemAdviceProvider> = Process::new(
            Kernel::new(&[]),
            StackInputs::new(vec![]),
            AdviceInputs::default().into(),
        );
        let program_outputs = process.execute(&program).expect("execution failed");
        let result_stack = program_outputs.stack();

        let g = Felt::get_root_of_unity(power);
        let one = g.exp(trace_len - 1).as_int();
        let two = g.exp(trace_len - 2).as_int();

        // results are in stack-order
        let expected = vec![two, one];
        assert!(
            result_stack
                .iter()
                .zip(expected.iter())
                .all(|(l, r)| l == r),
            "results don't match result={:?} expected={:?}",
            result_stack,
            expected,
        );
    }
}
