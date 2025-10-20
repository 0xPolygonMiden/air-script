use std::marker::PhantomData;

use p3_challenger::{HashChallenger, SerializingChallenger64};
use p3_circle::CirclePcs;
use p3_commit::ExtensionMmcs;
use p3_field::{PrimeField64, extension::BinomialExtensionField};
use p3_fri::create_benchmark_fri_params;
use p3_goldilocks::Goldilocks;
use p3_matrix::dense::RowMajorMatrix;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_sha256::Sha256;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher};
use p3_uni_stark::StarkConfig;

use crate::{
    generate_air_plonky3_test,
    helpers::check_constraints_with_periodic_columns,
    indexed_trace_access::indexed_trace_access_plonky3::{NUM_COLUMNS, TraceAccessAir},
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
    rows[0][2] = F::ZERO;
    rows[0][3] = F::ZERO;

    // Fill subsequent rows using direct access to the rows array
    #[allow(clippy::needless_range_loop)]
    for i in 1..num_rows {
        // Update current row
        rows[i][0] = F::ONE;
        rows[i][1] = F::ZERO;
        rows[i][2] = F::ZERO;
        rows[i][3] = F::ZERO;
    }

    trace
}

fn generate_inputs() -> Vec<u32> {
    vec![1; 16]
}
use p3_field::PrimeCharacteristicRing;

generate_air_plonky3_test!(test_air_plonky3, TraceAccessAir);
