//! Cross-backend comparison test for the ListComprehension AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the ListComprehension AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::list_comprehension::{
        list_comprehension::{
            ListComprehensionAir as WinterfellListComprehensionAir, PublicInputs,
        },
        list_comprehension_plonky3::ListComprehensionAir as Plonky3ListComprehensionAir,
    },
};

/// Configuration for ListComprehension AIR cross-backend comparison tests.
struct ListComprehensionTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl ListComprehensionTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut trace = vec![vec![Felt::ZERO; length]; 16];

        for row in 0..length {
            trace[1][row] = Felt::new(20);
            trace[14][row] = Felt::new(10);
            if row > 0 {
                trace[3][row] = Felt::new(2);
            }
        }

        trace
    }
}

impl CrossBackendTestConfig for ListComprehensionTestConfig {
    type WinterfellAir = WinterfellListComprehensionAir;
    type Plonky3Air = Plonky3ListComprehensionAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        16
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
    ) -> WinterfellListComprehensionAir {
        WinterfellListComprehensionAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3ListComprehensionAir {
        Plonky3ListComprehensionAir
    }
}

#[test]
fn test_list_comprehension_air_constraint_comparison() {
    let config = ListComprehensionTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "ListComprehension AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_list_comprehension_air_constraint_comparison_random_inputs() {
    let config = ListComprehensionTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_list_comprehension_air_constraint_comparison_random_inputs",
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

    println!("ListComprehension AIR random comparison passed for all 51 iterations (0-50)");
}
