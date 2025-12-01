use p3_field::PrimeField64;
use p3_miden_air::RowMajorMatrix;

use crate::{
    generate_air_plonky3_test_with_airscript_traits,
    test_utils::plonky3_traits::check_constraints_with_airscript_traits,
    tests::constants::constants_plonky3::{ConstantsAir, MAIN_WIDTH},
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
    rows[0][0] = F::ONE;
    rows[0][1] = F::ONE;
    rows[0][2] = F::ZERO;
    rows[0][3] = F::ONE;
    rows[0][4] = F::ONE;
    rows[0][5] = F::ZERO;
    rows[0][6] = F::ZERO;

    // Fill subsequent rows using direct access to the rows array
    for i in 1..num_rows {
        let col_0_prev = rows[i - 1][0];
        let col_1_prev = rows[i - 1][1];
        let col_2_prev = rows[i - 1][2];
        let col_3_prev = rows[i - 1][3];
        let col_4_prev = rows[i - 1][4];
        let col_5_prev = rows[i - 1][5];
        let col_6_prev = rows[i - 1][6];

        // Update current row based on previous values
        rows[i][0] = col_0_prev + F::ONE;
        rows[i][1] = F::ZERO;
        rows[i][2] = col_2_prev;
        rows[i][3] = col_3_prev;
        rows[i][4] = col_4_prev;
        rows[i][5] = col_5_prev + F::ONE;
        rows[i][6] = col_6_prev;
    }

    trace
}

fn generate_inputs() -> Vec<u32> {
    vec![1; 32]
}

generate_air_plonky3_test_with_airscript_traits!(test_air_plonky3, ConstantsAir);
