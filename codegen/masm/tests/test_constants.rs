use air_codegen_masm::code_gen;
use assembly::Assembler;
use ir::AirIR;
use processor::{
    math::{Felt, FieldElement},
    AdviceInputs, Kernel, MemAdviceProvider, Process, QuadExtension, StackInputs,
};

mod utils;
use utils::{parse, test_code, to_stack_order};

static CONSTANTS_AIR: &str = "
def ConstantsAir

const A = 2
const B = [3, 5]
const C = [[7, 11], [13, 17]]

trace_columns:
    main: [a, b, c]

public_inputs:
    stack_inputs: [16]

boundary_constraints:
    enf a.first = A
    enf b.first = A + B[0] * C[0][1]
    enf c.last = A - B[1] * C[0][0]

integrity_constraints:
    enf a' = a + A
    enf b' = B[0] * b
    enf c' = (C[0][0] + B[0]) * c
";
const A: QuadExtension<Felt> = QuadExtension::new(Felt::new(2), Felt::ZERO);
const B_0: QuadExtension<Felt> = QuadExtension::new(Felt::new(3), Felt::ZERO);
const C_0_0: QuadExtension<Felt> = QuadExtension::new(Felt::new(7), Felt::ZERO);

#[test]
fn test_constants() {
    let ast = parse(CONSTANTS_AIR);
    let ir = AirIR::new(ast).expect("build AirIR failed");
    let code = code_gen(&ir).expect("codegen failed");
    let trace_len = 2u64.pow(4);
    let z = QuadExtension::new(Felt::new(1), Felt::ZERO);

    let a = QuadExtension::new(Felt::new(19), Felt::ZERO);
    let b = QuadExtension::new(Felt::new(23), Felt::ZERO);
    let c = QuadExtension::new(Felt::new(29), Felt::ZERO);
    let a_prime = a + A;
    let b_prime = B_0 * b;
    let c_prime = (C_0_0 + B_0) * c;
    let main_frame = to_stack_order(&[a, a_prime, b, b_prime, c, c_prime]);
    let aux_frame = to_stack_order(&[]);
    let code = test_code(code, main_frame, aux_frame, trace_len, z);
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
