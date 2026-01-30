//! Cross-backend comparison test for the ListComprehensionNested AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the ListComprehensionNested AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::list_comprehension::{
        list_comprehension_nested::{
            ListComprehensionAir as WinterfellListComprehensionNestedAir, PublicInputs,
        },
        list_comprehension_nested_plonky3::ListComprehensionAir as Plonky3ListComprehensionNestedAir,
    },
};

/// Configuration for ListComprehensionNested AIR cross-backend comparison tests.
///
/// Note: Based on the AIR, it is apparently unsatisfiable at row 0 (boundary enforces
/// a0 = 0 while integrity equations imply a0 = 1, a1 = 1). This comparison test checks
/// that both backends evaluate constraints identically, not that the trace satisfies them.
struct ListComprehensionNestedTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl ListComprehensionNestedTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;

        vec![vec![Felt::ZERO; length]; 2]
    }
}

impl CrossBackendTestConfig for ListComprehensionNestedTestConfig {
    type WinterfellAir = WinterfellListComprehensionNestedAir;
    type Plonky3Air = Plonky3ListComprehensionNestedAir;
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
    ) -> WinterfellListComprehensionNestedAir {
        WinterfellListComprehensionNestedAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3ListComprehensionNestedAir {
        Plonky3ListComprehensionNestedAir
    }
}

#[test]
fn test_list_comprehension_nested_air_constraint_comparison() {
    let config = ListComprehensionNestedTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "ListComprehensionNested AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_list_comprehension_nested_air_constraint_comparison_random_inputs() {
    let config = ListComprehensionNestedTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_list_comprehension_nested_air_constraint_comparison_random_inputs",
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

    println!("ListComprehensionNested AIR random comparison passed for all 51 iterations (0-50)");
}
