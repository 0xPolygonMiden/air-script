use std::marker::PhantomData;

use p3_challenger::{HashChallenger, SerializingChallenger64};
use p3_circle::CirclePcs;
use p3_commit::ExtensionMmcs;
use p3_field::{PrimeCharacteristicRing, PrimeField64, extension::BinomialExtensionField};
use p3_fri::create_benchmark_fri_params;
use p3_goldilocks::Goldilocks;
use p3_matrix::dense::RowMajorMatrix;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_sha256::Sha256;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher};
use p3_uni_stark::StarkConfig;

use crate::{
    generate_air_plonky3_test_with_airscript_traits,
    test_utils::plonky3_traits::check_constraints_with_airscript_traits,
    tests::bitwise::bitwise_plonky3::{BitwiseAir, MAIN_WIDTH},
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

    let first = F::ZERO;

    // Initialize first row
    rows[0][0] = first;
    rows[0][1] = first;
    rows[0][2] = first;
    rows[0][3] = first;
    rows[0][4] = first;
    rows[0][5] = first;
    rows[0][6] = first;
    rows[0][7] = first;
    rows[0][8] = first;
    rows[0][9] = first;
    rows[0][10] = first;
    rows[0][11] = first;
    rows[0][12] = first;
    rows[0][13] = first;

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
        let col_12_prev = rows[i - 1][12];
        let col_13_prev = rows[i - 1][13];

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
        rows[i][12] = col_12_prev;
        rows[i][13] = col_13_prev;
    }

    trace
}

fn generate_inputs() -> Vec<u32> {
    vec![1; 16]
}

generate_air_plonky3_test_with_airscript_traits!(test_air_plonky3, BitwiseAir);
