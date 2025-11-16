// Helper macros for test generation

/// Generates an AIR test function with the standard boilerplate
///
/// # Arguments
/// * `test_name` - The identifier for the test function (e.g., `test_binary_air`)
/// * `air_name` - The identifier for the AIR struct (e.g., `BinaryAir`)
/// * `tester_name` - The identifier for the `AirTester` struct (e.g., `BinaryAirTester`)
/// * `trace_length` - The length of the trace for the test (e.g., `32` or `1024`)
#[macro_export]
macro_rules! generate_air_test {
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
