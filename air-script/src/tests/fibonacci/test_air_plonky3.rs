use p3_field::PrimeField64;
use p3_miden_air::RowMajorMatrix;

use crate::{
    generate_air_plonky3_test_with_airscript_traits,
    test_utils::plonky3_traits::check_constraints_with_airscript_traits,
    tests::fibonacci::fibonacci_plonky3::{FibonacciAir, MAIN_WIDTH},
};

pub fn generate_trace_rows<F: PrimeField64>(inputs: Vec<u32>) -> RowMajorMatrix<F> {
    let num_rows = 31;
    let trace_length = num_rows * MAIN_WIDTH;

    let mut long_trace = F::zero_vec(trace_length);

    let mut trace = RowMajorMatrix::new(long_trace, MAIN_WIDTH);

    let (prefix, rows, suffix) = unsafe { trace.values.align_to_mut::<[F; MAIN_WIDTH]>() };
    assert!(prefix.is_empty(), "Alignment should match");
    assert!(suffix.is_empty(), "Alignment should match");
    assert_eq!(rows.len(), num_rows);

    // Initialize first row
    rows[0][0] = F::from_canonical_checked(inputs[0]).unwrap();
    rows[0][1] = F::from_canonical_checked(inputs[1]).unwrap();

    // Fill subsequent rows using direct access to the rows array
    for i in 1..num_rows {
        let cur_a = rows[i - 1][0];
        let cur_b = rows[i - 1][1];

        // Update current row based on previous values
        rows[i][0] = cur_b;
        rows[i][1] = cur_a + cur_b;
    }

    trace
}

fn generate_inputs() -> Vec<u32> {
    let one = 1;
    let last = 2178309; // 32nd Fibonacci number
    vec![one, one, last]
}

generate_air_plonky3_test_with_airscript_traits!(test_air_plonky3, FibonacciAir);
