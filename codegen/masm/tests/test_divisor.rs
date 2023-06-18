use air_codegen_masm::{code_gen, constants};
use assembly::Assembler;
use ir::AirIR;
use processor::{
    math::{Felt, FieldElement},
    AdviceInputs, Kernel, MemAdviceProvider, Process, QuadExtension, StackInputs,
};
use winter_prover::ConstraintDivisor;

mod utils;
use utils::{parse, test_code, to_stack_order, Data};

static SIMPLE_AIR: &str = "
def SimpleAux

trace_columns:
    main: [a]

public_inputs:
    stack_inputs: [1]

boundary_constraints:
    enf a.first = 0

integrity_constraints:
    enf a = 0
";

#[test]
fn test_integrity_divisor() {
    let ast = parse(SIMPLE_AIR);
    let ir = AirIR::new(ast).expect("build AirIR failed");
    let code = code_gen(&ir).expect("codegen failed");

    let exemptions = 2;
    let one = QuadExtension::new(Felt::new(1), Felt::ZERO);
    let a = QuadExtension::new(Felt::new(3), Felt::ZERO);
    let a_prime = a;
    let z = QuadExtension::new(Felt::new(5), Felt::new(7));

    for power in 3..32 {
        let trace_len = 2u64.pow(power);

        let code = test_code(
            code.clone(),
            vec![
                Data {
                    data: to_stack_order(&[a, a_prime]),
                    address: constants::OOD_FRAME_ADDRESS,
                    descriptor: "main_trace",
                },
                Data {
                    data: to_stack_order(&vec![one; 6]),
                    address: constants::COMPOSITION_COEF_ADDRESS,
                    descriptor: "composition_coefficients",
                },
            ],
            trace_len,
            z,
            &["cache_z_exp", "compute_integrity_constraint_divisor"],
        );
        let program = Assembler::default().compile(code).unwrap();

        let constraint_divisor =
            ConstraintDivisor::<Felt>::from_transition(trace_len.try_into().unwrap(), exemptions);
        let divisor = constraint_divisor.evaluate_at(z);

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
            divisor,
        ]);

        assert!(
            result_stack
                .iter()
                .zip(expected.iter())
                .all(|(l, r)| l == r),
            "results don't match trace_len={} power={} result={:?} expected={:?}",
            trace_len,
            power,
            result_stack,
            expected,
        );
    }
}
