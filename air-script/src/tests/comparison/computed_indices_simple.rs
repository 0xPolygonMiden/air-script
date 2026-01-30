//! Cross-backend comparison test for the ComputedIndicesSimple AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the ComputedIndicesSimple AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::computed_indices::{
        computed_indices_simple::{
            ComputedIndicesAir as WinterfellComputedIndicesAir, PublicInputs,
        },
        computed_indices_simple_plonky3::ComputedIndicesAir as Plonky3ComputedIndicesAir,
    },
};

/// Configuration for ComputedIndicesSimple AIR cross-backend comparison tests.
struct ComputedIndicesSimpleTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl ComputedIndicesSimpleTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;

        let col0 = vec![Felt::ZERO; length];
        let col1 = vec![Felt::new(2); length];
        let col2 = vec![Felt::new(4); length];
        let col3 = vec![Felt::new(6); length];
        let col4 = vec![Felt::ZERO; length];
        let col5 = vec![Felt::ZERO; length];
        let col6 = vec![Felt::ZERO; length];
        let col7 = vec![Felt::ZERO; length];

        vec![col0, col1, col2, col3, col4, col5, col6, col7]
    }
}

impl CrossBackendTestConfig for ComputedIndicesSimpleTestConfig {
    type WinterfellAir = WinterfellComputedIndicesAir;
    type Plonky3Air = Plonky3ComputedIndicesAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        8
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
    ) -> WinterfellComputedIndicesAir {
        WinterfellComputedIndicesAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3ComputedIndicesAir {
        Plonky3ComputedIndicesAir
    }
}

#[test]
fn test_computed_indices_simple_air_constraint_comparison() {
    let config = ComputedIndicesSimpleTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "ComputedIndicesSimple AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_computed_indices_simple_air_constraint_comparison_random_inputs() {
    let config = ComputedIndicesSimpleTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_computed_indices_simple_air_constraint_comparison_random_inputs",
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

    println!("ComputedIndicesSimple AIR random comparison passed for all 51 iterations (0-50)");
}
