use air_codegen_masm::{code_gen, constants};
use assembly::Assembler;
use ir::AirIR;
use processor::{
    math::{Felt, FieldElement, StarkField},
    AdviceInputs, Kernel, MemAdviceProvider, Process, QuadExtension, StackInputs,
};
use winter_prover::{Assertion, ConstraintDivisor};

mod utils;
use utils::{parse, test_code, to_stack_order, Data};

static SIMPLE_INTEGRITY_AIR: &str = "
def SimpleIntegrityAux

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
    let ast = parse(SIMPLE_INTEGRITY_AIR);
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
                    data: to_stack_order(&vec![one; 2]),
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

static SIMPLE_BOUNDARY_AIR: &str = "
def SimpleBoundaryAux

trace_columns:
    main: [a]
    aux: [b]

public_inputs:
    stack_inputs: [1]

boundary_constraints:
    enf a.first = 0
    enf a.last = 0
    enf b.first = 0
    enf b.last = 0

integrity_constraints:
    enf a = 0
";

#[test]
fn test_boundary_divisor() {
    let ast = parse(SIMPLE_BOUNDARY_AIR);
    let ir = AirIR::new(ast).expect("build AirIR failed");
    let code = code_gen(&ir).expect("codegen failed");

    let exemptions = 2;
    let one = QuadExtension::new(Felt::new(1), Felt::ZERO);
    let a = QuadExtension::new(Felt::new(13), Felt::ZERO);
    let a_prime = a;
    let a_column = 0;
    let b = QuadExtension::new(Felt::new(17), Felt::ZERO);
    let b_prime = b;
    let b_column = 1;
    let z = QuadExtension::new(Felt::new(19), Felt::new(23));

    for power in 3..32 {
        let trace_len = 2u64.pow(power);
        let first_step = 0;
        let last_step: usize = (trace_len - exemptions).try_into().unwrap();

        let code = test_code(
            code.clone(),
            vec![
                Data {
                    data: to_stack_order(&[a, a_prime]),
                    address: constants::OOD_FRAME_ADDRESS,
                    descriptor: "main_trace",
                },
                Data {
                    data: to_stack_order(&[b, b_prime]),
                    address: constants::OOD_AUX_FRAME_ADDRESS,
                    descriptor: "aux_trace",
                },
                Data {
                    data: to_stack_order(&vec![one; 5]),
                    address: constants::COMPOSITION_COEF_ADDRESS,
                    descriptor: "composition_coefficients",
                },
            ],
            trace_len,
            z,
            &[
                "cache_z_exp",
                // The exemption point is cached as part of the integrity constraint computation
                "compute_integrity_constraint_divisor",
                "evaluate_boundary_constraints",
            ],
        );
        println!("\n{}", code);
        let program = Assembler::default().compile(code).unwrap();

        let a_first_assertion = Assertion::<Felt>::single(a_column, first_step, Felt::new(3));
        let a_last_assertion = Assertion::<Felt>::single(a_column, last_step, Felt::new(5));
        let b_first_assertion = Assertion::<Felt>::single(b_column, first_step, Felt::new(7));
        let b_last_assertion = Assertion::<Felt>::single(b_column, last_step, Felt::new(11));

        let mut result = QuadExtension::ZERO;
        for (assertion, numerator) in [
            (a_first_assertion, a),
            (a_last_assertion, a_prime),
            (b_first_assertion, b),
            (b_last_assertion, b_prime),
        ] {
            let divisor = ConstraintDivisor::<Felt>::from_assertion(
                &assertion,
                trace_len.try_into().unwrap(),
            );
            println!(
                "numerator={:?}",
                numerator
                    .to_base_elements()
                    .iter()
                    .map(|e| e.as_int())
                    .collect::<Vec<u64>>()
            );
            println!(
                "divisor={:?}",
                divisor
                    .evaluate_at(z)
                    .to_base_elements()
                    .iter()
                    .map(|e| e.as_int())
                    .collect::<Vec<u64>>()
            );
            result += numerator / divisor.evaluate_at(z);
        }

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
            result,
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
