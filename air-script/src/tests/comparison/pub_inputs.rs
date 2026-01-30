//! Cross-backend comparison test for the PubInputs AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the PubInputs AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::pub_inputs::{
        pub_inputs::{PubInputsAir as WinterfellPubInputsAir, PublicInputs},
        pub_inputs_plonky3::PubInputsAir as Plonky3PubInputsAir,
    },
};

/// Configuration for PubInputs AIR cross-backend comparison tests.
struct PubInputsTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl PubInputsTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut trace = vec![vec![Felt::ZERO; length]; 4];

        for row in 0..length {
            trace[1][row] = Felt::new(2);
            trace[2][row] = Felt::new(3);
            if row > 0 {
                trace[0][row] = Felt::new(5);
            }
        }

        trace
    }
}

impl CrossBackendTestConfig for PubInputsTestConfig {
    type WinterfellAir = WinterfellPubInputsAir;
    type Plonky3Air = Plonky3PubInputsAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        4
    }

    fn trace_length(&self) -> usize {
        self.trace_length
    }

    fn build_winterfell_trace(&self) -> Vec<Vec<Felt>> {
        self.build_trace()
    }

    fn build_winterfell_public_inputs(&self) -> PublicInputs {
        PublicInputs::new([Felt::ZERO; 4], [Felt::ZERO; 4], [Felt::ZERO; 4], [Felt::ZERO; 20])
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellPubInputsAir {
        WinterfellPubInputsAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3PubInputsAir {
        Plonky3PubInputsAir
    }
}

#[test]
fn test_pub_inputs_air_constraint_comparison() {
    let config = PubInputsTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "PubInputs AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_pub_inputs_air_constraint_comparison_random_inputs() {
    let config = PubInputsTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_pub_inputs_air_constraint_comparison_random_inputs",
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

    println!("PubInputs AIR random comparison passed for all 51 iterations (0-50)");
}
