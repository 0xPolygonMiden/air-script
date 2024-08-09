use air_codegen_masm::constants;
use miden_assembly::Assembler;
use miden_processor::{
    math::{Felt, FieldElement},
    AdviceInputs, Kernel, MemAdviceProvider, Process, QuadExtension, StackInputs,
};

mod utils;
use utils::{codegen, test_code, to_stack_order, Data};

static CONSTANTS_AIR: &str = "
def ConstantsAir

const A = 2;
const B = [3, 5];
const C = [[7, 11], [13, 17]];

trace_columns {
    main: [a, b, c],
}

public_inputs {
    stack_inputs: [16],
}

boundary_constraints {
    enf a.first = A;
    enf b.first = A + B[0] * C[0][1];
    enf c.last = A - B[1] * C[0][0];
}

integrity_constraints {
    enf a' = a + A;
    enf b' = B[0] * b;
    enf c' = (C[0][0] + B[0]) * c;
}";
const A: QuadExtension<Felt> = QuadExtension::new(Felt::new(2), Felt::ZERO);
const B_0: QuadExtension<Felt> = QuadExtension::new(Felt::new(3), Felt::ZERO);
const C_0_0: QuadExtension<Felt> = QuadExtension::new(Felt::new(7), Felt::ZERO);

#[test]
fn test_constants() {
    let code = codegen(CONSTANTS_AIR);

    let trace_len = 2u64.pow(4);
    let one = QuadExtension::new(Felt::new(1), Felt::ZERO);
    let z = one;
    let a = QuadExtension::new(Felt::new(19), Felt::ZERO);
    let b = QuadExtension::new(Felt::new(23), Felt::ZERO);
    let c = QuadExtension::new(Felt::new(29), Felt::ZERO);
    let a_prime = a + A;
    let b_prime = B_0 * b;
    let c_prime = (C_0_0 + B_0) * c;

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
        &["compute_integrity_constraints"],
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
        QuadExtension::<Felt>::ZERO,
        QuadExtension::<Felt>::ZERO,
        QuadExtension::<Felt>::ZERO,
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
