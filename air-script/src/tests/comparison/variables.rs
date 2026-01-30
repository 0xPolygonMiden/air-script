//! Cross-backend comparison test for the Variables AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the Variables AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::variables::{
        variables::{PublicInputs, VariablesAir as WinterfellVariablesAir},
        variables_plonky3::VariablesAir as Plonky3VariablesAir,
    },
};

/// Configuration for Variables AIR cross-backend comparison tests.
struct VariablesTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl VariablesTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut trace = vec![vec![Felt::ZERO; length]; 4];

        for row in 0..length {
            trace[0][row] = Felt::ONE;
            if row > 0 {
                trace[1][row] = Felt::ONE;
            }
        }

        trace
    }
}

impl CrossBackendTestConfig for VariablesTestConfig {
    type WinterfellAir = WinterfellVariablesAir;
    type Plonky3Air = Plonky3VariablesAir;
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
        PublicInputs::new([Felt::ZERO; 16], [Felt::ZERO; 16])
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellVariablesAir {
        WinterfellVariablesAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3VariablesAir {
        Plonky3VariablesAir
    }

    fn periodic_column_values(&self) -> Vec<Vec<u64>> {
        vec![vec![1, 1, 1, 1, 1, 1, 1, 0]]
    }
}

#[test]
fn test_variables_air_constraint_comparison() {
    let config = VariablesTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "Variables AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_variables_air_constraint_comparison_random_inputs() {
    let config = VariablesTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_variables_air_constraint_comparison_random_inputs",
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

    println!("Variables AIR random comparison passed for all 51 iterations (0-50)");
}
