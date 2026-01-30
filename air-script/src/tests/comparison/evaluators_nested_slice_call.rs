//! Cross-backend comparison test for the EvaluatorsNestedSliceCall AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the EvaluatorsNestedSliceCall AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::evaluators::{
        evaluators_nested_slice_call::{
            EvaluatorsSliceAir as WinterfellEvaluatorsNestedSliceCallAir, PublicInputs,
        },
        evaluators_nested_slice_call_plonky3::EvaluatorsSliceAir as Plonky3EvaluatorsNestedSliceCallAir,
    },
};

/// Configuration for EvaluatorsNestedSliceCall AIR cross-backend comparison tests.
struct EvaluatorsNestedSliceCallTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl EvaluatorsNestedSliceCallTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;

        vec![vec![Felt::ZERO; length]; 20]
    }
}

impl CrossBackendTestConfig for EvaluatorsNestedSliceCallTestConfig {
    type WinterfellAir = WinterfellEvaluatorsNestedSliceCallAir;
    type Plonky3Air = Plonky3EvaluatorsNestedSliceCallAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        20
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
    ) -> WinterfellEvaluatorsNestedSliceCallAir {
        WinterfellEvaluatorsNestedSliceCallAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3EvaluatorsNestedSliceCallAir {
        Plonky3EvaluatorsNestedSliceCallAir
    }
}

#[test]
fn test_evaluators_nested_slice_call_air_constraint_comparison() {
    let config = EvaluatorsNestedSliceCallTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "EvaluatorsNestedSliceCall AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_evaluators_nested_slice_call_air_constraint_comparison_random_inputs() {
    let config = EvaluatorsNestedSliceCallTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_evaluators_nested_slice_call_air_constraint_comparison_random_inputs",
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

    println!("EvaluatorsNestedSliceCall AIR random comparison passed for all 51 iterations (0-50)");
}
