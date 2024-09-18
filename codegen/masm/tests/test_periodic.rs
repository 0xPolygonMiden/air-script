use air_codegen_masm::constants;
use miden_assembly::Assembler;
use miden_processor::{
    math::{Felt, FieldElement},
    AdviceInputs, Kernel, MemAdviceProvider, Process, QuadExtension, StackInputs,
};

mod utils;
use utils::{codegen, test_code, to_stack_order, Data};

static SIMPLE_AUX_AIR: &str = "
def SimpleAux

trace_columns {
    main: [a],
}

periodic_columns {
    k: [1, 1],
}

public_inputs {
    stack_inputs: [16],
}

boundary_constraints {
    enf a.first = 0;
}

integrity_constraints {
    enf a * k = 0;
}";

#[test]
fn test_simple_periodic() {
    let code = codegen(SIMPLE_AUX_AIR);

    let trace_len = 2u64.pow(4);
    let one = QuadExtension::new(Felt::new(1), Felt::ZERO);
    let z = one;
    let a = QuadExtension::new(Felt::new(3), Felt::ZERO);
    let a_prime = a;

    let code = test_code(
        code,
        vec![
            Data {
                data: to_stack_order(&[a, a_prime]),
                address: constants::OOD_FRAME_ADDRESS,
                descriptor: "main_trace",
            },
            Data {
                data: to_stack_order(&[]),
                address: constants::OOD_AUX_FRAME_ADDRESS,
                descriptor: "aux_trace",
            },
            Data {
                data: to_stack_order(&[one; 1]),
                address: constants::COMPOSITION_COEF_ADDRESS,
                descriptor: "composition_coefficients",
            },
        ],
        trace_len,
        z,
        &[
            "cache_z_exp",
            "cache_periodic_polys",
            "compute_integrity_constraints",
        ],
    );
    let program = Assembler::default().compile(code).unwrap();

    let mut process: Process<MemAdviceProvider> = Process::new(
        Kernel::new(&[]),
        StackInputs::new(vec![]),
        AdviceInputs::default().into(),
    );
    let program_outputs = process.execute(&program).expect("execution failed");
    let result_stack = program_outputs.stack();

    // results are in stack-order
    #[rustfmt::skip]
    let expected = to_stack_order(&[
        a,
    ]);

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

static MULTIPLE_AUX_AIR: &str = "
def MultipleAux

trace_columns {
    main: [a, b, c],
}

periodic_columns {
    m: [1, 0],
    n: [1, 1, 1, 0],
    o: [1, 0, 0, 0],
}

public_inputs {
    stack_inputs: [16],
}

boundary_constraints {
    enf a.first = 0;
}

integrity_constraints {
    enf a * m = 0;
    enf b * n = 0;
    enf c * o = 0;
}";

#[test]
fn test_multiple_periodic() {
    let code = codegen(MULTIPLE_AUX_AIR);

    let trace_len = 2u64.pow(4);
    let one = QuadExtension::new(Felt::new(1), Felt::ZERO);
    let z = one;
    let a = QuadExtension::new(Felt::new(3), Felt::ZERO);
    let a_prime = a;
    let b = QuadExtension::new(Felt::new(5), Felt::ZERO);
    let b_prime = b;
    let c = QuadExtension::new(Felt::new(7), Felt::ZERO);
    let c_prime = c;

    let code = test_code(
        code,
        vec![
            Data {
                data: to_stack_order(&[a, a_prime, b, b_prime, c, c_prime]),
                address: constants::OOD_FRAME_ADDRESS,
                descriptor: "main_trace",
            },
            Data {
                data: to_stack_order(&[]),
                address: constants::OOD_AUX_FRAME_ADDRESS,
                descriptor: "aux_trace",
            },
            Data {
                data: to_stack_order(&[one; 3]),
                address: constants::COMPOSITION_COEF_ADDRESS,
                descriptor: "composition_coefficients",
            },
        ],
        trace_len,
        z,
        &[
            "cache_z_exp",
            "cache_periodic_polys",
            "compute_integrity_constraints",
        ],
    );
    let program = Assembler::default().compile(code).unwrap();

    let mut process: Process<MemAdviceProvider> = Process::new(
        Kernel::new(&[]),
        StackInputs::new(vec![]),
        AdviceInputs::default().into(),
    );
    let program_outputs = process.execute(&program).expect("execution failed");
    let result_stack = program_outputs.stack();

    // results are in stack-order
    #[rustfmt::skip]
    let expected = to_stack_order(&[
        c,
        b,
        a,
    ]);

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
