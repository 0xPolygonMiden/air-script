use p3_field::PrimeField64;
use p3_miden_air::RowMajorMatrix;

use crate::{
    generate_air_plonky3_test_with_airscript_traits,
    tests::constant_in_range::constant_in_range_plonky3::{ConstantInRangeAir, MAIN_WIDTH},
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
    rows[0][0] = F::from_canonical_checked(3).unwrap();

    // Fill subsequent rows using direct access to the rows array
    for i in 1..num_rows {
        let col_0_prev = rows[i - 1][0];
        let col_1_prev = rows[i - 1][1];
        let col_2_prev = rows[i - 1][2];
        let col_3_prev = rows[i - 1][3];
        let col_4_prev = rows[i - 1][4];
        let col_5_prev = rows[i - 1][5];
        let col_6_prev = rows[i - 1][6];
        let col_7_prev = rows[i - 1][7];
        let col_8_prev = rows[i - 1][8];
        let col_9_prev = rows[i - 1][9];
        let col_10_prev = rows[i - 1][10];
        let col_11_prev = rows[i - 1][11];

        // Update current row based on previous values
        rows[i][0] = col_0_prev;
        rows[i][1] = col_1_prev;
        rows[i][2] = col_2_prev;
        rows[i][3] = col_3_prev;
        rows[i][4] = col_4_prev;
        rows[i][5] = col_5_prev;
        rows[i][6] = col_6_prev;
        rows[i][7] = col_7_prev;
        rows[i][8] = col_8_prev;
        rows[i][9] = col_9_prev;
        rows[i][10] = col_10_prev;
        rows[i][11] = col_11_prev;
    }

    trace
}

fn generate_inputs() -> Vec<u64> {
    vec![1; 16]
}

generate_air_plonky3_test_with_airscript_traits!(test_air_plonky3, ConstantInRangeAir);
