//! Cross-backend comparison test for the Binary AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the Binary AIR at every row of the trace.

use p3_field::PrimeCharacteristicRing;
use p3_goldilocks::Goldilocks;
use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{
        CrossBackendTestConfig, run_cross_backend_comparison, run_cross_backend_comparison_random,
    },
    tests::binary::{
        binary::{BinaryAir as WinterfellBinaryAir, PublicInputs},
        binary_plonky3::BinaryAir as Plonky3BinaryAir,
    },
};

// ============================================================================
// Test Configuration
// ============================================================================

/// Configuration for Binary AIR cross-backend comparison tests.
struct BinaryTestConfig {
    /// The starting value for the binary trace (0 or 1).
    start_value: u64,
    /// The trace length.
    trace_length: usize,
}

impl BinaryTestConfig {
    fn new(start_value: u64, trace_length: usize) -> Self {
        Self { start_value, trace_length }
    }

    /// Build a trace for the Binary AIR.
    ///
    /// The Binary AIR has 2 columns (a, b) that alternate between 0 and 1:
    /// - Row 0: (start, start)
    /// - Row 1: (1-start, 1-start)
    /// - Row 2: (start, start)
    /// - ...
    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut col_a = vec![Felt::ZERO; length];
        let mut col_b = vec![Felt::ZERO; length];

        let start = Felt::new(self.start_value);
        let one = Felt::ONE;

        col_a[0] = start;
        col_b[0] = start;

        for i in 1..length {
            col_a[i] = one - col_a[i - 1];
            col_b[i] = one - col_b[i - 1];
        }

        vec![col_a, col_b]
    }
}

impl CrossBackendTestConfig for BinaryTestConfig {
    type WinterfellAir = WinterfellBinaryAir;
    type Plonky3Air = Plonky3BinaryAir;
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
        let mut inputs = [Felt::ZERO; 16];
        inputs[0] = Felt::new(self.start_value);
        PublicInputs::new(inputs)
    }

    fn build_plonky3_public_inputs(&self) -> Vec<Goldilocks> {
        (0..16)
            .map(|i| {
                if i == 0 {
                    Goldilocks::from_u64(self.start_value)
                } else {
                    Goldilocks::ZERO
                }
            })
            .collect()
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellBinaryAir {
        WinterfellBinaryAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3BinaryAir {
        Plonky3BinaryAir
    }

    fn num_public_values(&self) -> usize {
        16
    }
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_binary_air_constraint_comparison() {
    let config = BinaryTestConfig::new(0, 64);
    let result = run_cross_backend_comparison(&config);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "Binary AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_binary_air_constraint_comparison_start_one() {
    let config = BinaryTestConfig::new(1, 64);
    let result = run_cross_backend_comparison(&config);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "Binary AIR comparison (start=1) passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_binary_air_constraint_comparison_random_inputs() {
    let config = BinaryTestConfig::new(0, 64);

    // Test with iterations 0 through 50 for thorough coverage
    for iteration in 0u64..=50 {
        let result = run_cross_backend_comparison_random(
            &config,
            "test_binary_air_constraint_comparison_random_inputs",
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

    println!("Binary AIR random comparison passed for all 51 iterations (0-50)");
}
