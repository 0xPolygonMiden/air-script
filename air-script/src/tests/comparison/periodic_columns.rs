//! Cross-backend comparison test for the PeriodicColumns AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the PeriodicColumns AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::periodic_columns::{
        periodic_columns::{PeriodicColumnsAir as WinterfellPeriodicColumnsAir, PublicInputs},
        periodic_columns_plonky3::PeriodicColumnsAir as Plonky3PeriodicColumnsAir,
    },
};

/// Configuration for PeriodicColumns AIR cross-backend comparison tests.
struct PeriodicColumnsTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl PeriodicColumnsTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;

        vec![vec![Felt::ZERO; length]; 3]
    }
}

impl CrossBackendTestConfig for PeriodicColumnsTestConfig {
    type WinterfellAir = WinterfellPeriodicColumnsAir;
    type Plonky3Air = Plonky3PeriodicColumnsAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        3
    }

    fn trace_length(&self) -> usize {
        self.trace_length
    }

    fn build_winterfell_trace(&self) -> Vec<Vec<Felt>> {
        self.build_trace()
    }

    fn build_winterfell_public_inputs(&self) -> PublicInputs {
        PublicInputs::new([Felt::ZERO; 16])
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellPeriodicColumnsAir {
        WinterfellPeriodicColumnsAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3PeriodicColumnsAir {
        Plonky3PeriodicColumnsAir
    }

    fn periodic_column_values(&self) -> Vec<Vec<u64>> {
        vec![vec![1, 0, 0, 0], vec![1, 1, 1, 1, 1, 1, 1, 0]]
    }
}

#[test]
fn test_periodic_columns_air_constraint_comparison() {
    let config = PeriodicColumnsTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "PeriodicColumns AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_periodic_columns_air_constraint_comparison_random_inputs() {
    let config = PeriodicColumnsTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_periodic_columns_air_constraint_comparison_random_inputs",
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

    println!("PeriodicColumns AIR random comparison passed for all 51 iterations (0-50)");
}
