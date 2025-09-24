use std::marker::PhantomData;

use p3_challenger::{HashChallenger, SerializingChallenger32};
use p3_circle::CirclePcs;
use p3_commit::ExtensionMmcs;
use p3_field::{PrimeField64, extension::BinomialExtensionField};
use p3_fri::create_benchmark_fri_params;
use p3_matrix::dense::RowMajorMatrix;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_mersenne_31::Mersenne31;
use p3_sha256::Sha256;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher};
use p3_uni_stark::StarkConfig;

use crate::{
    constants::constants_plonky3::{ConstantsAir, NUM_COLUMNS},
    helpers::check_constraints_with_periodic_columns,
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

#[test]
fn test_air_plonky3() {
    type Val = Mersenne31;
    type Challenge = BinomialExtensionField<Val, 3>;

    type ByteHash = Sha256;
    type FieldHash = SerializingHasher<ByteHash>;
    type MyCompress = CompressionFunctionFromHasher<ByteHash, 2, 32>;
    type ValMmcs = MerkleTreeMmcs<Val, u8, FieldHash, MyCompress, 32>;
    type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
    type Challenger = SerializingChallenger32<Val, HashChallenger<u8, ByteHash, 32>>;
    type Pcs = CirclePcs<Val, ValMmcs, ChallengeMmcs>;
    type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;

    let byte_hash = ByteHash {};
    let field_hash = FieldHash::new(Sha256);
    let compress = MyCompress::new(byte_hash);
    let val_mmcs = ValMmcs::new(field_hash, compress);
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
    let challenger = Challenger::from_hasher(vec![], byte_hash);
    let fri_params = create_benchmark_fri_params(challenge_mmcs);
    let pcs = Pcs {
        mmcs: val_mmcs,
        fri_params,
        _phantom: PhantomData,
    };
    let config = MyConfig::new(pcs, challenger);

    let inputs = vec![1; 32];
    let inputs_m31: Vec<Val> = inputs.iter().map(|&x| Val::new_checked(x).unwrap()).collect();

    let trace = generate_trace_rows::<Val>(inputs);

    check_constraints_with_periodic_columns(&ConstantsAir {}, &trace, &inputs_m31);
}
