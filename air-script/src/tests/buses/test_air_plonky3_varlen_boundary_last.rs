use p3_field::PrimeField64;
use p3_miden_air::RowMajorMatrix;

use crate::{
    generate_air_plonky3_test_with_airscript_traits,
    tests::buses::buses_varlen_boundary_last_plonky3::{BusesAir, MAIN_WIDTH},
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
    rows[0][0] = F::ONE;
    rows[0][1] = F::ZERO;
    rows[0][2] = F::ZERO;
    rows[0][3] = F::ZERO;
    rows[0][4] = F::ZERO;

    // Fill subsequent rows using direct access to the rows array
    for i in 1..num_rows {
        let a_prev = rows[i - 1][0];
        let b_prev = rows[i - 1][1];
        let c_prev = rows[i - 1][2];
        let d_prev = rows[i - 1][3];
        let e_prev = rows[i - 1][4];

        // Update current row based on previous values
        rows[i][0] = F::ONE;
        rows[i][1] = if i > 3 && i < 8 { F::ONE } else { F::ZERO }; // sp_insert is true 4 times
        rows[i][2] = if i > 3 && i < 7 { F::ONE } else { F::ZERO }; // sp_remove is true 3 times
        rows[i][3] = if i > 4 && i < 10 { F::ONE } else { F::ZERO }; // sq_insert_twice is true 5 times
        rows[i][4] = if i > 5 && i < 10 {
            F::from_canonical_checked(2).unwrap()
        } else {
            F::ZERO
        }; // sq_remove has value "2" 4 times
    }

    trace
}

fn generate_inputs() -> Vec<u64> {
    vec![]
}

fn generate_var_len_pub_inputs<'a>() -> Vec<Vec<Vec<u64>>> {
    // At the end, the bus p will have the tuple (a) (that equals (1)) inserted once
    let var_len_p = vec![vec![1]];
    // At the end, the bus q will have the tuple (2, a) (that equals (2, 1)) inserted twice
    let var_len_q = vec![vec![2, 1], vec![2, 1]];
    vec![var_len_p, var_len_q]
}

generate_air_plonky3_test_with_airscript_traits!(test_air_plonky3, BusesAir);
