use air_codegen_masm::constants;
use miden_assembly::Assembler;
use miden_processor::{
    math::{Felt, FieldElement},
    AdviceInputs, Kernel, MemAdviceProvider, Process, QuadExtension, StackInputs,
};

mod utils;
use utils::{codegen, test_code, to_stack_order, Data};

static ARITH_AIR: &str = "
def SimpleArithmetic

trace_columns {
    main: [a, b],
}

public_inputs {
    stack_inputs: [16],
}

boundary_constraints {
    enf a.first = 0;
}

integrity_constraints {
    enf a + a = 0;
    enf a - a = 0;
    enf a * a = 0;

    enf b + a = 0;
    enf b - a = 0;
    enf b * a = 0;
}";

#[test]
fn test_simple_arithmetic() {
    let code = codegen(ARITH_AIR);

    let trace_len = 2u64.pow(4);
    let one = QuadExtension::new(Felt::new(1), Felt::ZERO);
    let z = one;
    let a = QuadExtension::new(Felt::new(3), Felt::ZERO);
    let b = QuadExtension::new(Felt::new(7), Felt::ZERO);
    let a_prime = a;
    let b_prime = b;

    let code = test_code(
        code,
        vec![
            Data {
                data: to_stack_order(&[a, a_prime, b, b_prime]),
                address: constants::OOD_FRAME_ADDRESS,
                descriptor: "main_trace",
            },
            Data {
                data: to_stack_order(&[]),
                address: constants::OOD_AUX_FRAME_ADDRESS,
                descriptor: "aux_trace",
            },
            Data {
                data: to_stack_order(&[one; 6]),
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
        b * a,
        b - a,
        b + a,
        a * a,
        a - a,
        a + a,
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

static EXP_AIR: &str = "
def Exp

trace_columns {
    main: [a, b],
}

public_inputs {
    stack_inputs: [16],
}

boundary_constraints {
    enf a.first = 0;
}

integrity_constraints {
    enf b^1 = 0;
    enf b^2 = 0;
    enf b^3 = 0;
    enf b^4 = 0;
    enf b^5 = 0;
}";

#[test]
fn test_exp() {
    let code = codegen(EXP_AIR);

    let trace_len = 2u64.pow(4);
    let one = QuadExtension::new(Felt::new(1), Felt::ZERO);
    let z = one;
    let a = QuadExtension::<Felt>::ZERO;
    let b = QuadExtension::new(Felt::new(3), Felt::ZERO);
    let a_prime = a;
    let b_prime = b;

    let code = test_code(
        code,
        vec![
            Data {
                data: to_stack_order(&[a, a_prime, b, b_prime]),
                address: constants::OOD_FRAME_ADDRESS,
                descriptor: "main_trace",
            },
            Data {
                data: to_stack_order(&[]),
                address: constants::OOD_AUX_FRAME_ADDRESS,
                descriptor: "aux_trace",
            },
            Data {
                data: to_stack_order(&[one; 5]),
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
        b.exp(5),
        b.exp(4),
        b.exp(3),
        b.exp(2),
        b.exp(1),
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

static LONG_TRACE: &str = "
def LongTrace

trace_columns {
    main: [a, b, c, d, e, f, g, h, i],
}

public_inputs {
    stack_inputs: [16],
}

boundary_constraints {
    enf a.first = 0;
}

integrity_constraints {
    enf a * b * c + d - e = 0;
}";

#[test]
fn test_long_trace() {
    let code = codegen(LONG_TRACE);

    let trace_len = 2u64.pow(4);
    let one = QuadExtension::new(Felt::new(1), Felt::ZERO);
    let z = one;
    let a = QuadExtension::new(Felt::new(2), Felt::ZERO);
    let b = QuadExtension::new(Felt::new(3), Felt::ZERO);
    let c = QuadExtension::new(Felt::new(5), Felt::ZERO);
    let d = QuadExtension::new(Felt::new(7), Felt::ZERO);
    let e = QuadExtension::new(Felt::new(11), Felt::ZERO);
    let a_prime = a;
    let b_prime = b;
    let c_prime = c;
    let d_prime = d;
    let e_prime = e;

    let code = test_code(
        code,
        vec![
            Data {
                data: to_stack_order(&[a, a_prime, b, b_prime, c, c_prime, d, d_prime, e, e_prime]),
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

    let expected = to_stack_order(&[a * b * c + d - e]);
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

static VECTOR: &str = "
def Vector

trace_columns {
    main: [clk, fmp[2]],
}

public_inputs {
    stack_inputs: [16],
}

boundary_constraints {
    enf clk.first = 0;
}

integrity_constraints {
    enf clk - fmp[0] + fmp[1] = 0;
}";

#[test]
fn test_vector() {
    let code = codegen(VECTOR);

    let trace_len = 2u64.pow(4);
    let one = QuadExtension::new(Felt::new(1), Felt::ZERO);
    let z = one;
    let clk = QuadExtension::new(Felt::new(2), Felt::ZERO);
    let fmp_0 = QuadExtension::new(Felt::new(3), Felt::ZERO);
    let fmp_1 = QuadExtension::new(Felt::new(5), Felt::ZERO);
    let clk_prime = clk;
    let fmp_0_prime = fmp_0;
    let fmp_1_prime = fmp_1;
    let main_frame = to_stack_order(&[clk, clk_prime, fmp_0, fmp_0_prime, fmp_1, fmp_1_prime]);
    let aux_frame = to_stack_order(&[]);

    let code = test_code(
        code,
        vec![
            Data {
                data: main_frame,
                address: constants::OOD_FRAME_ADDRESS,
                descriptor: "main_trace",
            },
            Data {
                data: aux_frame,
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

    let expected = to_stack_order(&[clk - fmp_0 + fmp_1]);
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

static MULTIPLE_ROWS_AIR: &str = "
def MultipleRows

trace_columns {
    main: [a, b],
}

public_inputs {
    stack_inputs: [16],
}

boundary_constraints {
    enf a.first = 0;
}

integrity_constraints {
    enf a' = a * 2;
    enf b' = a + b;
}";

#[test]
fn test_multiple_rows() {
    let code = codegen(MULTIPLE_ROWS_AIR);

    let trace_len = 2u64.pow(4);
    let one = QuadExtension::new(Felt::new(1), Felt::ZERO);
    let z = one;
    let two = QuadExtension::new(Felt::new(2), Felt::ZERO);
    let a = QuadExtension::new(Felt::new(3), Felt::ZERO);
    let b = QuadExtension::new(Felt::new(7), Felt::ZERO);
    let a_prime = a * two;
    let b_prime = a + b;

    let code = test_code(
        code,
        vec![
            Data {
                data: to_stack_order(&[a, a_prime, b, b_prime]),
                address: constants::OOD_FRAME_ADDRESS,
                descriptor: "main_trace",
            },
            Data {
                data: to_stack_order(&[]),
                address: constants::OOD_AUX_FRAME_ADDRESS,
                descriptor: "aux_trace",
            },
            Data {
                data: to_stack_order(&[one; 2]),
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
