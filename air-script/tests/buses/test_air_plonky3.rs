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
    buses::buses_complex_plonky3::{BusesAir, NUM_COLUMNS},
    generate_air_plonky3_test_with_airscript_traits,
    helpers::check_constraints_with_airscript_traits,
};

pub fn generate_trace_rows<F: PrimeField64>(inputs: Vec<u32>) -> RowMajorMatrix<F> {
    let num_rows = 32;
    let trace_length = num_rows * NUM_COLUMNS;

    let mut long_trace = F::zero_vec(trace_length);

    let mut trace = RowMajorMatrix::new(long_trace, NUM_COLUMNS);

    let (prefix, rows, suffix) = unsafe { trace.values.align_to_mut::<[F; NUM_COLUMNS]>() };
    assert!(prefix.is_empty(), "Alignment should match");
    assert!(suffix.is_empty(), "Alignment should match");
    assert_eq!(rows.len(), num_rows);

    // Initialize first row
    rows[0][0] = F::ZERO;
    rows[0][1] = F::ZERO;

    // Fill subsequent rows using direct access to the rows array
    for i in 1..num_rows {
        let a_prev = rows[i - 1][0];
        let b_prev = rows[i - 1][1];

        // Update current row based on previous values
        rows[i][0] = F::ONE - a_prev;
        rows[i][1] = F::ONE - b_prev;
    }

    trace
}

fn generate_inputs() -> Vec<u32> {
    vec![1; 2]
}

generate_air_plonky3_test_with_airscript_traits!(test_air_plonky3, BusesAir);
