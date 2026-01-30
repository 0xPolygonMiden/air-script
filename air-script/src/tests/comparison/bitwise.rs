//! Cross-backend comparison test for the Bitwise AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the Bitwise AIR at every row of the trace.
//!
//! The Bitwise AIR is important because it uses **periodic columns**:
//! - k0: `[1, 0, 0, 0, 0, 0, 0, 0]` (period 8) - active on rows 0, 8, 16, ...
//! - k1: `[1, 1, 1, 1, 1, 1, 1, 0]` (period 8) - inactive only on rows 7, 15, 23, ...
//!
//! This test validates that the periodic column evaluation works correctly
//! in the cross-backend comparison framework.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{
        CrossBackendTestConfig, run_cross_backend_comparison, run_cross_backend_comparison_random,
    },
    tests::bitwise::{
        bitwise::{BitwiseAir as WinterfellBitwiseAir, PublicInputs},
        bitwise_plonky3::BitwiseAir as Plonky3BitwiseAir,
    },
};

/// Configuration for Bitwise AIR cross-backend comparison tests.
struct BitwiseTestConfig {
    /// The trace length (must be divisible by the period 8).
    trace_length: usize,
}

impl BitwiseTestConfig {
    fn new(trace_length: usize) -> Self {
        // Trace length should be divisible by 8 (the periodic column period)
        assert!(trace_length % 8 == 0, "Trace length must be divisible by 8 for bitwise AIR");
        Self { trace_length }
    }

    /// Build a trace for the Bitwise AIR.
    ///
    /// The Bitwise AIR has 14 columns. For testing, we use an all-zeros trace
    /// which satisfies all the constraints (binary checks, decomposition, etc.).
    ///
    /// The constraints include:
    /// - Binary checks: col[i]^2 - col[i] = 0 for binary columns
    /// - Decomposition constraints using periodic column k0
    /// - Transition constraints using periodic column k1
    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let trace_width = 14;

        // Initialize all columns to zero
        // This satisfies all binary constraints (0^2 - 0 = 0)
        // and most other constraints (they become 0 * something = 0)
        let trace: Vec<Vec<Felt>> = vec![vec![Felt::ZERO; length]; trace_width];

        // Column 13 must be 0 at first row (boundary constraint)
        // Already zero, so nothing to do

        trace
    }
}

impl CrossBackendTestConfig for BitwiseTestConfig {
    type WinterfellAir = WinterfellBitwiseAir;
    type Plonky3Air = Plonky3BitwiseAir;
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
    ) -> WinterfellBitwiseAir {
        WinterfellBitwiseAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3BitwiseAir {
        Plonky3BitwiseAir
    }

    /// Returns the periodic column values for the Bitwise AIR.
    ///
    /// Two periodic columns with period 8:
    /// - k0: `[1, 0, 0, 0, 0, 0, 0, 0]` - marks the first row of each 8-row block
    /// - k1: `[1, 1, 1, 1, 1, 1, 1, 0]` - active on all rows except the last of each block
    fn periodic_column_values(&self) -> Vec<Vec<u64>> {
        vec![
            vec![1, 0, 0, 0, 0, 0, 0, 0], // k0
            vec![1, 1, 1, 1, 1, 1, 1, 0], // k1
        ]
    }
}

#[test]
fn test_bitwise_air_constraint_comparison() {
    // Use 64 rows (8 complete periods)
    let config = BitwiseTestConfig::new(64);
    let result = run_cross_backend_comparison(&config);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "Bitwise AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_bitwise_air_constraint_comparison_larger_trace() {
    // Use 512 rows (64 complete periods) - same as the Winterfell test
    let config = BitwiseTestConfig::new(512);
    let result = run_cross_backend_comparison(&config);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "Bitwise AIR comparison (512 rows) passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_bitwise_air_constraint_comparison_random_inputs() {
    let config = BitwiseTestConfig::new(64);

    // Test with iterations 0 through 50 for thorough coverage
    for iteration in 0u64..=50 {
        let result = run_cross_backend_comparison_random(
            &config,
            "test_bitwise_air_constraint_comparison_random_inputs",
            iteration,
        );

        if !result.is_ok() {
            panic!(
                "Random constraint evaluation comparison failed (iteration={})!\n\n{}",
                iteration,
                result.format_report()
            );
        }
    }

    println!("Bitwise AIR random comparison passed for all 51 iterations (0-50)");
}
