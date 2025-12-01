use p3_field::PrimeField64;
use p3_miden_air::RowMajorMatrix;

use crate::{
    generate_air_plonky3_test_with_airscript_traits,
    test_utils::plonky3_traits::check_constraints_with_airscript_traits,
    tests::buses::buses_complex_plonky3::{BusesAir, MAIN_WIDTH},
};

pub fn generate_trace_rows<F: PrimeField64>(inputs: Vec<u32>) -> RowMajorMatrix<F> {
    let num_rows = 32;
    let trace_length = num_rows * MAIN_WIDTH;

    let mut long_trace = F::zero_vec(trace_length);

    let mut trace = RowMajorMatrix::new(long_trace, MAIN_WIDTH);

    let (prefix, rows, suffix) = unsafe { trace.values.align_to_mut::<[F; MAIN_WIDTH]>() };
    assert!(prefix.is_empty(), "Alignment should match");
    assert!(suffix.is_empty(), "Alignment should match");
    assert_eq!(rows.len(), num_rows);

    // Initialize first row
    rows[0][0] = F::ZERO;
    rows[0][1] = F::ZERO;
    rows[0][2] = F::ZERO;
    rows[0][3] = F::ZERO;
    rows[0][4] = F::ZERO;
    rows[0][5] = F::ZERO;
    rows[0][6] = F::ZERO;

    // Fill subsequent rows using direct access to the rows array
    for i in 1..num_rows {
        let a_prev = rows[i - 1][0];
        let b_prev = rows[i - 1][1];
        let c_prev = rows[i - 1][2];
        let d_prev = rows[i - 1][3];
        let e_prev = rows[i - 1][4];
        let f_prev = rows[i - 1][5];
        let g_prev = rows[i - 1][6];

        // Update current row based on previous values
        rows[i][0] = F::ZERO;
        rows[i][1] = F::ZERO;
        rows[i][2] = if i > 3 && i < 8 { F::ONE } else { F::ZERO }; // s1 is true 4 times
        rows[i][3] = if i > 5 && i < 10 { F::ONE } else { F::ZERO }; // s2 is true 4 times
        rows[i][4] = if i > 4 && i < 10 { F::ONE } else { F::ZERO }; // s3 is true 5 times
        rows[i][5] = if i > 5 && i < 13 { F::ONE } else { F::ZERO }; // s4 is true 7 times
        rows[i][6] = if i > 15 && i < 20 { F::from_u64(3) } else { F::ZERO }; // d is set to 3 four times
    }

    trace
}

fn generate_inputs() -> Vec<u32> {
    vec![1; 2]
}

generate_air_plonky3_test_with_airscript_traits!(test_air_plonky3, BusesAir);
