//! Cross-backend comparison test for the TraceColGroups AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the TraceColGroups AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::trace_col_groups::{
        trace_col_groups::{PublicInputs, TraceColGroupAir as WinterfellTraceColGroupAir},
        trace_col_groups_plonky3::TraceColGroupAir as Plonky3TraceColGroupAir,
    },
};

/// Configuration for TraceColGroups AIR cross-backend comparison tests.
struct TraceColGroupsTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl TraceColGroupsTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut trace = vec![vec![Felt::ZERO; length]; 9];

        for row in 1..length {
            trace[1][row] = trace[1][row - 1] - Felt::ONE;
            trace[2][row] = trace[2][row - 1] + Felt::ONE;
        }

        trace
    }
}

impl CrossBackendTestConfig for TraceColGroupsTestConfig {
    type WinterfellAir = WinterfellTraceColGroupAir;
    type Plonky3Air = Plonky3TraceColGroupAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        9
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
    ) -> WinterfellTraceColGroupAir {
        WinterfellTraceColGroupAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3TraceColGroupAir {
        Plonky3TraceColGroupAir
    }
}

#[test]
fn test_trace_col_groups_air_constraint_comparison() {
    let config = TraceColGroupsTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "TraceColGroups AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_trace_col_groups_air_constraint_comparison_random_inputs() {
    let config = TraceColGroupsTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_trace_col_groups_air_constraint_comparison_random_inputs",
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

    println!("TraceColGroups AIR random comparison passed for all 51 iterations (0-50)");
}
