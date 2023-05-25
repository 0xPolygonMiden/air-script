use air_codegen_masm::code_gen;
use assembly::Assembler;
use ir::AirIR;
use processor::{
    math::{Felt, FieldElement},
    AdviceInputs, Kernel, MemAdviceProvider, Process, QuadExtension, StackInputs,
};

mod utils;
use utils::{parse, test_code, to_stack_order};

static SIMPLE_AUX_AIR: &str = "
def SimpleAux

trace_columns:
    main: [a]

periodic_columns:
    k: [1, 1]

public_inputs:
    stack_inputs: [16]

boundary_constraints:
    enf a.first = 0

integrity_constraints:
    enf a * k = 0
";

#[test]
fn test_simple_periodic() {
    let ast = parse(SIMPLE_AUX_AIR);
    let ir = AirIR::new(ast).expect("build AirIR failed");
    let code = code_gen(&ir).expect("codegen failed");
    let trace_len = 2u64.pow(4);
    let z = QuadExtension::new(Felt::new(1), Felt::ZERO);

    let a = QuadExtension::new(Felt::new(3), Felt::ZERO);
    let a_prime = a;
    let main_frame = to_stack_order(&[a, a_prime]);
    let aux_frame = to_stack_order(&[]);
    let code = test_code(
        code,
        main_frame,
        aux_frame,
        trace_len,
        z,
        &["cache_periodic_polys", "compute_evaluate_transitions"],
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
