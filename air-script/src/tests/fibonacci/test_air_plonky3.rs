use p3_field::PrimeField64;
use p3_miden_air::RowMajorMatrix;

use crate::{
    generate_air_plonky3_test_with_airscript_traits,
    tests::fibonacci::fibonacci_plonky3::{FibonacciAir, MAIN_WIDTH},
};

pub fn generate_trace_rows<F: PrimeField64>(inputs: Vec<u64>) -> RowMajorMatrix<F> {
    let num_rows = 512;
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

fn generate_inputs() -> Vec<u64> {
    let zero = 0;
    let one = 1;
    let last = fibonacci_field::<p3_goldilocks::Goldilocks>(512).as_canonical_u64(); // 512nd Fibonacci number in Goldilock's field
    vec![zero, one, last]
}

fn fibonacci_field<F: PrimeField64>(n: i32) -> F {
    if n < 0 {
        panic!("{} is negative!", n);
    } else if n == 0 {
        return F::ZERO;
    } else if n == 1 {
        return F::ONE;
    }

    let mut sum = F::ZERO;
    let mut last = F::ZERO;
    let mut curr = F::ONE;
    for _i in 1..n {
        sum = last + curr;
        last = curr;
        curr = sum;
    }
    sum
}

#[test]
fn test_goldilocks_fibonacci_computation() {
    type F = p3_goldilocks::Goldilocks;
    let f_32 = fibonacci_field::<F>(32);
    let f_512 = fibonacci_field::<F>(512);
    assert_eq!(f_32, F::new(2178309));
    assert_eq!(f_512, F::new(12556846397060607923));
}

generate_air_plonky3_test_with_airscript_traits!(test_air_plonky3, FibonacciAir);
