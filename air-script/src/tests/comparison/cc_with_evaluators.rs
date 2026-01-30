//! Cross-backend comparison test for the ConstraintComprehension AIR (evaluator variant).
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the evaluator-based constraint comprehension AIR.
//!
//! Note: This AIR shares generated outputs with `constraint_comprehension`.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::constraint_comprehension::{
        constraint_comprehension::{
            ConstraintComprehensionAir as WinterfellConstraintComprehensionAir, PublicInputs,
        },
        constraint_comprehension_plonky3::ConstraintComprehensionAir as Plonky3ConstraintComprehensionAir,
    },
};

/// Configuration for ConstraintComprehension (with evaluators) cross-backend comparison tests.
struct ConstraintComprehensionWithEvaluatorsTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl ConstraintComprehensionWithEvaluatorsTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut trace: Vec<Vec<Felt>> = vec![vec![Felt::ZERO; length]; 14];

        for row in 0..length {
            trace[0][row] = Felt::new(row as u64);
            trace[1][row] = Felt::new(row as u64 * 2);
            trace[2][row] = Felt::new(row as u64 * 3);
            trace[3][row] = Felt::new(100);
            trace[4][row] = Felt::new(row as u64 + 10);
            trace[5][row] = Felt::new(row as u64 + 20);

            trace[6][row] = Felt::new(row as u64 + 1);
            trace[7][row] = Felt::new(row as u64 + 2);
            trace[8][row] = if row == 0 {
                Felt::ZERO
            } else {
                Felt::new(row as u64 + 3)
            };
            trace[9][row] = Felt::new(row as u64 + 4);

            trace[10][row] = trace[6][row];
            trace[11][row] = trace[7][row];
            trace[12][row] = trace[8][row];
            trace[13][row] = trace[9][row];
        }

        trace
    }
}

impl CrossBackendTestConfig for ConstraintComprehensionWithEvaluatorsTestConfig {
    type WinterfellAir = WinterfellConstraintComprehensionAir;
    type Plonky3Air = Plonky3ConstraintComprehensionAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        14
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
    ) -> WinterfellConstraintComprehensionAir {
        WinterfellConstraintComprehensionAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3ConstraintComprehensionAir {
        Plonky3ConstraintComprehensionAir
    }
}

#[test]
fn test_cc_with_evaluators_air_constraint_comparison() {
    let config = ConstraintComprehensionWithEvaluatorsTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "ConstraintComprehension (with evaluators) comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_cc_with_evaluators_air_constraint_comparison_random_inputs() {
    let config = ConstraintComprehensionWithEvaluatorsTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_cc_with_evaluators_air_constraint_comparison_random_inputs",
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
        "ConstraintComprehension (with evaluators) random comparison passed for all 51 iterations (0-50)"
    );
}
