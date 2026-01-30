//! Cross-backend comparison test for the ListFolding AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the ListFolding AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::list_folding::{
        list_folding::{ListFoldingAir as WinterfellListFoldingAir, PublicInputs},
        list_folding_plonky3::ListFoldingAir as Plonky3ListFoldingAir,
    },
};

/// Configuration for ListFolding AIR cross-backend comparison tests.
struct ListFoldingTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl ListFoldingTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut trace = vec![vec![Felt::ZERO; length]; 17];

        for row in 1..length {
            trace[3][row] = Felt::new(2);
        }

        trace
    }
}

impl CrossBackendTestConfig for ListFoldingTestConfig {
    type WinterfellAir = WinterfellListFoldingAir;
    type Plonky3Air = Plonky3ListFoldingAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        17
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
    ) -> WinterfellListFoldingAir {
        WinterfellListFoldingAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3ListFoldingAir {
        Plonky3ListFoldingAir
    }
}

#[test]
fn test_list_folding_air_constraint_comparison() {
    let config = ListFoldingTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "ListFolding AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_list_folding_air_constraint_comparison_random_inputs() {
    let config = ListFoldingTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_list_folding_air_constraint_comparison_random_inputs",
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

    println!("ListFolding AIR random comparison passed for all 51 iterations (0-50)");
}
