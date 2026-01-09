//! Cross-backend comparison test for the ConstraintComprehension AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the ConstraintComprehension AIR at every row of the trace.
//!
//! The ConstraintComprehension AIR tests list comprehension in constraints:
//! - Boundary: `c[2].first = 0` (column 8 at row 0)
//! - Integrity: `c = d for (c, d) in (c, d)` expands to c[i] = d[i] for i in 0..4
//!
//! Trace columns: [clk, fmp[2], ctx, a, b, c[4], d[4]] (14 total)
//! Column indices: clk=0, fmp=1-2, ctx=3, a=4, b=5, c=6-9, d=10-13

use p3_field::PrimeCharacteristicRing;
use p3_goldilocks::Goldilocks;
use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{
        CrossBackendTestConfig, run_cross_backend_comparison, run_cross_backend_comparison_random,
    },
    tests::constraint_comprehension::{
        constraint_comprehension::{
            ConstraintComprehensionAir as WinterfellConstraintComprehensionAir, PublicInputs,
        },
        constraint_comprehension_plonky3::ConstraintComprehensionAir as Plonky3ConstraintComprehensionAir,
    },
};

// ============================================================================
// Test Configuration
// ============================================================================

/// Configuration for ConstraintComprehension AIR cross-backend comparison tests.
struct ConstraintComprehensionTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl ConstraintComprehensionTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    /// Build a trace for the ConstraintComprehension AIR.
    ///
    /// The trace has 14 columns: [clk, fmp[2], ctx, a, b, c[4], d[4]]
    /// - Indices: clk=0, fmp=1-2, ctx=3, a=4, b=5, c=6-9, d=10-13
    ///
    /// Constraints:
    /// - Boundary: c[2].first = 0 → column 8 at row 0 must be 0
    /// - Integrity: c[i] = d[i] for all rows and i in 0..4
    ///
    /// We build a meaningful trace where c and d have matching non-zero values.
    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;

        // Initialize all 14 columns
        let mut trace: Vec<Vec<Felt>> = vec![vec![Felt::ZERO; length]; 14];

        // Fill with meaningful values
        for row in 0..length {
            // clk increments
            trace[0][row] = Felt::new(row as u64);

            // fmp[0], fmp[1] - some values
            trace[1][row] = Felt::new(row as u64 * 2);
            trace[2][row] = Felt::new(row as u64 * 3);

            // ctx
            trace[3][row] = Felt::new(100);

            // a, b - some values
            trace[4][row] = Felt::new(row as u64 + 10);
            trace[5][row] = Felt::new(row as u64 + 20);

            // c[0..4] - columns 6-9
            // c[2] (column 8) must be 0 at first row due to boundary constraint
            trace[6][row] = Felt::new(row as u64 + 1); // c[0]
            trace[7][row] = Felt::new(row as u64 + 2); // c[1]
            trace[8][row] = if row == 0 {
                Felt::ZERO // c[2] must be 0 at first row
            } else {
                Felt::new(row as u64 + 3)
            };
            trace[9][row] = Felt::new(row as u64 + 4); // c[3]

            // d[0..4] - columns 10-13
            // Must equal c[0..4] due to integrity constraint: c[i] = d[i]
            trace[10][row] = trace[6][row]; // d[0] = c[0]
            trace[11][row] = trace[7][row]; // d[1] = c[1]
            trace[12][row] = trace[8][row]; // d[2] = c[2]
            trace[13][row] = trace[9][row]; // d[3] = c[3]
        }

        trace
    }
}

impl CrossBackendTestConfig for ConstraintComprehensionTestConfig {
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

    fn build_plonky3_public_inputs(&self) -> Vec<Goldilocks> {
        vec![Goldilocks::ZERO; 16]
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

    fn num_public_values(&self) -> usize {
        16
    }
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_constraint_comprehension_air_constraint_comparison() {
    let config = ConstraintComprehensionTestConfig::new(64);
    let result = run_cross_backend_comparison(&config);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "ConstraintComprehension AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_constraint_comprehension_air_constraint_comparison_small_trace() {
    let config = ConstraintComprehensionTestConfig::new(16);
    let result = run_cross_backend_comparison(&config);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "ConstraintComprehension AIR comparison (16 rows) passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_constraint_comprehension_air_constraint_comparison_random_inputs() {
    let config = ConstraintComprehensionTestConfig::new(64);

    // Test with iterations 0 through 50 for thorough coverage
    for iteration in 0u64..=50 {
        let result = run_cross_backend_comparison_random(
            &config,
            "test_constraint_comprehension_air_constraint_comparison_random_inputs",
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

    println!("ConstraintComprehension AIR random comparison passed for all 51 iterations (0-50)");
}
