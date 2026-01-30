//! Cross-backend comparison test for the IndexedTraceAccess AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the IndexedTraceAccess AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::indexed_trace_access::{
        indexed_trace_access::{PublicInputs, TraceAccessAir as WinterfellTraceAccessAir},
        indexed_trace_access_plonky3::TraceAccessAir as Plonky3TraceAccessAir,
    },
};

/// Configuration for IndexedTraceAccess AIR cross-backend comparison tests.
struct IndexedTraceAccessTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl IndexedTraceAccessTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;

        let mut col0 = vec![Felt::ZERO; length];
        for row in 1..length {
            col0[row] = Felt::ONE;
        }

        let col1 = vec![Felt::ZERO; length];
        let col2 = vec![Felt::ZERO; length];
        let col3 = vec![Felt::ZERO; length];

        vec![col0, col1, col2, col3]
    }
}

impl CrossBackendTestConfig for IndexedTraceAccessTestConfig {
    type WinterfellAir = WinterfellTraceAccessAir;
    type Plonky3Air = Plonky3TraceAccessAir;
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
        PublicInputs::new([Felt::ZERO; 16])
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellTraceAccessAir {
        WinterfellTraceAccessAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3TraceAccessAir {
        Plonky3TraceAccessAir
    }
}

#[test]
fn test_indexed_trace_access_air_constraint_comparison() {
    let config = IndexedTraceAccessTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "IndexedTraceAccess AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_indexed_trace_access_air_constraint_comparison_random_inputs() {
    let config = IndexedTraceAccessTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_indexed_trace_access_air_constraint_comparison_random_inputs",
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

    println!("IndexedTraceAccess AIR random comparison passed for all 51 iterations (0-50)");
}
