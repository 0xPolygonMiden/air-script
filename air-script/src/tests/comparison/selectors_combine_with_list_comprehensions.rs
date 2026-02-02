//! Cross-backend comparison test for the SelectorsCombineWithListComprehensions AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the SelectorsCombineWithListComprehensions AIR at every row of the
//! trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::selectors::{
        selectors_combine_with_list_comprehensions::{
            PublicInputs, SelectorsAir as WinterfellSelectorsCombineWithListComprehensionsAir,
        },
        selectors_combine_with_list_comprehensions_plonky3::SelectorsAir as Plonky3SelectorsCombineWithListComprehensionsAir,
    },
};

/// Configuration for SelectorsCombineWithListComprehensions AIR cross-backend comparison tests.
struct SelectorsCombineWithListComprehensionsTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl SelectorsCombineWithListComprehensionsTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut trace = vec![vec![Felt::ZERO; length]; 6];

        for row in 0..length {
            trace[4][row] = Felt::new(8);
            if row > 0 {
                trace[5][row] = Felt::new(8);
            }
        }

        trace
    }
}

impl CrossBackendTestConfig for SelectorsCombineWithListComprehensionsTestConfig {
    type WinterfellAir = WinterfellSelectorsCombineWithListComprehensionsAir;
    type Plonky3Air = Plonky3SelectorsCombineWithListComprehensionsAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        6
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
    ) -> WinterfellSelectorsCombineWithListComprehensionsAir {
        WinterfellSelectorsCombineWithListComprehensionsAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3SelectorsCombineWithListComprehensionsAir {
        Plonky3SelectorsCombineWithListComprehensionsAir
    }
}

#[test]
fn test_selectors_combine_with_list_comprehensions_air_constraint_comparison() {
    let config = SelectorsCombineWithListComprehensionsTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "SelectorsCombineWithListComprehensions AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_selectors_combine_with_list_comprehensions_air_constraint_comparison_random_inputs() {
    let config = SelectorsCombineWithListComprehensionsTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_selectors_combine_with_list_comprehensions_air_constraint_comparison_random_inputs",
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
        "SelectorsCombineWithListComprehensions AIR random comparison passed for all 51 iterations (0-50)"
    );
}
