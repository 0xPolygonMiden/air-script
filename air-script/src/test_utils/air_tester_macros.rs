// Helper macros for Winterfell test generation

/// Generates a Winterfell AIR test function with the standard boilerplate
///
/// # Arguments
/// * `test_name` - The identifier for the test function (e.g., `test_binary_air`)
/// * `air_name` - The identifier for the AIR struct (e.g., `BinaryAir`)
/// * `tester_name` - The identifier for the `AirTester` struct (e.g., `BinaryAirTester`)
/// * `trace_length` - The length of the trace for the test (e.g., `32` or `1024`)
#[macro_export]
macro_rules! generate_air_winterfell_test {
    ($test_name:ident, $air_name:path, $tester_name:ident, $trace_length:expr) => {
        #[test]
        fn $test_name() {
            use winter_math::fields::f64::BaseElement as Felt;
            let air_tester = Box::new($tester_name {});
            let length = $trace_length;

            let main_trace = air_tester.build_main_trace(length);
            let aux_trace = air_tester.build_aux_trace(length);
            let pub_inputs = air_tester.public_inputs();
            let trace_info = air_tester.build_trace_info(length);
            let options = air_tester.build_proof_options();

            let air = <$air_name>::new(trace_info, pub_inputs, options);
            main_trace.validate::<$air_name, Felt>(&air, aux_trace.as_ref());
        }
    };
}

// Helper macros for Plonky3 test generation

/// Generates a Plonky3 AIR test function with the standard boilerplate
///
/// # Arguments
/// * `test_name` - The identifier for the test function (e.g., `test_binary_air`)
/// * `air_name` - The identifier for the AIR struct (e.g., `BinaryAir`)
#[macro_export]
macro_rules! generate_air_plonky3_test_with_airscript_traits {
    ($test_name:ident, $air_name:ident) => {
        #[test]
        fn $test_name() {
            type Val = p3_goldilocks::Goldilocks;
            type Challenge = p3_field::extension::BinomialExtensionField<Val, 2>;
            type ByteHash = p3_sha256::Sha256;
            type FieldHash = p3_symmetric::SerializingHasher<ByteHash>;
            type MyCompress = p3_symmetric::CompressionFunctionFromHasher<ByteHash, 2, 32>;
            type ValMmcs = p3_merkle_tree::MerkleTreeMmcs<Val, u8, FieldHash, MyCompress, 32>;
            type ChallengeMmcs = p3_commit::ExtensionMmcs<Val, Challenge, ValMmcs>;
            type Challenger = p3_challenger::SerializingChallenger64<
                Val,
                p3_challenger::HashChallenger<u8, ByteHash, 32>,
            >;
            type Dft = p3_dft::Radix2DitParallel<Val>;
            type Pcs = p3_fri::TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;
            type MyConfig = p3_uni_stark::StarkConfig<Pcs, Challenge, Challenger>;

            let byte_hash = ByteHash {};
            let field_hash = FieldHash::new(p3_sha256::Sha256);
            let compress = MyCompress::new(byte_hash);
            let val_mmcs = ValMmcs::new(field_hash, compress);
            let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
            let challenger = Challenger::from_hasher(vec![], byte_hash);
            let dft = Dft::default();
            let fri_params = p3_fri::create_benchmark_fri_params(challenge_mmcs);
            let pcs = Pcs::new(dft, val_mmcs, fri_params);
            let config = MyConfig::new(pcs, challenger);

            let inputs = generate_inputs();
            let inputs_goldilocks: Vec<Val> = inputs
                .iter()
                .map(|&x| <Val as p3_field::PrimeCharacteristicRing>::from_u32(x))
                .collect();

            let trace = generate_trace_rows::<Val>(inputs);

            let proof = p3_miden_prover::prove(&config, &$air_name {}, &trace, &inputs_goldilocks);
            p3_miden_prover::verify(&config, &$air_name {}, &proof, &inputs_goldilocks)
                .expect("Verification failed");
        }
    };
}
