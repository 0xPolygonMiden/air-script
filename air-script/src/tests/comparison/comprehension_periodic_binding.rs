//! Cross-backend comparison test for the ComprehensionPeriodicBinding AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the ComprehensionPeriodicBinding AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::comprehension_periodic_binding::{
        comprehension_periodic_binding::{
            ComprehensionPeriodicBindingTest as WinterfellComprehensionPeriodicBindingTest,
            PublicInputs,
        },
        comprehension_periodic_binding_plonky3::ComprehensionPeriodicBindingTest as Plonky3ComprehensionPeriodicBindingTest,
    },
};

/// Configuration for ComprehensionPeriodicBinding AIR cross-backend comparison tests.
struct ComprehensionPeriodicBindingTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl ComprehensionPeriodicBindingTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;

        vec![vec![Felt::ZERO; length]; 2]
    }
}

impl CrossBackendTestConfig for ComprehensionPeriodicBindingTestConfig {
    type WinterfellAir = WinterfellComprehensionPeriodicBindingTest;
    type Plonky3Air = Plonky3ComprehensionPeriodicBindingTest;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        2
    }

    fn trace_length(&self) -> usize {
        self.trace_length
    }

    fn build_winterfell_trace(&self) -> Vec<Vec<Felt>> {
        self.build_trace()
    }

    fn build_winterfell_public_inputs(&self) -> PublicInputs {
        PublicInputs::new([Felt::ZERO; 1])
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellComprehensionPeriodicBindingTest {
        WinterfellComprehensionPeriodicBindingTest::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3ComprehensionPeriodicBindingTest {
        Plonky3ComprehensionPeriodicBindingTest
    }

    fn periodic_column_values(&self) -> Vec<Vec<u64>> {
        vec![vec![1, 2], vec![3, 4]]
    }
}

#[test]
fn test_comprehension_periodic_binding_air_constraint_comparison() {
    let config = ComprehensionPeriodicBindingTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "ComprehensionPeriodicBinding AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_comprehension_periodic_binding_air_constraint_comparison_random_inputs() {
    let config = ComprehensionPeriodicBindingTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_comprehension_periodic_binding_air_constraint_comparison_random_inputs",
                iteration,
            },
        );

        if !result.is_ok() {
            panic!(
                "Random constraint evaluation comparison failed (iteration={})!\n\n{}",
                iteration,
                result.format_report()
            );
        }
    }

    println!(
        "ComprehensionPeriodicBinding AIR random comparison passed for all 51 iterations (0-50)"
    );
}
