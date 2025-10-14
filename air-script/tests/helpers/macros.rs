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
macro_rules! generate_air_plonky3_test {
    ($test_name:ident, $air_name:ident) => {
        #[test]
        fn $test_name() {
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

            let inputs = generate_inputs();
            let inputs_m31: Vec<Val> =
                inputs.iter().map(|&x| Val::new_checked(x).unwrap()).collect();

            let trace = generate_trace_rows::<Val>(inputs);

            check_constraints_with_periodic_columns(&$air_name {}, &trace, &inputs_m31);

            /*let prove_with_periodic_columns = prove_with_periodic_columns(&config, &BitwiseAir {}, trace, &inputs_m31);
            verify_with_periodic_columns(&config, &BitwiseAir {}, &prove_with_periodic_columns, &inputs_m31).expect("Verification failed");*/

            /*let proof = prove(&config, &BitwiseAir {}, trace, &inputs_m31);
            verify(&config, &BitwiseAir {}, &proof, &inputs_m31).expect("Verification failed");*/
        }
    };
}
