//! Cross-backend comparison test for the Constants AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the Constants AIR at every row of the trace.
//!
//! The Constants AIR tests:
//! - `when_first_row()` boundary constraints (5 constraints)
//! - `when_last_row()` boundary constraints (1 constraint)
//! - `when_transition()` transition constraints (4 constraints)
//! - Global integrity constraints (1 constraint without `when_transition()`)

use p3_field::PrimeCharacteristicRing;
use p3_goldilocks::Goldilocks;
use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{
        CrossBackendTestConfig, run_cross_backend_comparison, run_cross_backend_comparison_random,
    },
    tests::constants::{
        constants::{ConstantsAir as WinterfellConstantsAir, PublicInputs},
        constants_plonky3::ConstantsAir as Plonky3ConstantsAir,
    },
};

/// Configuration for Constants AIR cross-backend comparison tests.
struct ConstantsTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl ConstantsTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    /// Build a trace for the Constants AIR.
    ///
    /// The trace has 7 columns with the following constraints:
    /// - Boundary (first row): col[0]=1, col[1]=1, col[2]=0, col[3]=1, col[4]=1
    /// - Boundary (last row): col[6]=0
    /// - Transition: col[0]' = col[0] + 1, col[1]' = 0, col[2]' = col[2], col[5]' = col[5] + 1
    /// - Integrity (global): col[4] = 1
    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;

        // Initialize columns
        let mut col0 = vec![Felt::ZERO; length]; // Increments by 1 each row
        let mut col1 = vec![Felt::ZERO; length]; // 1 at first row, 0 elsewhere
        let mut col2 = vec![Felt::ZERO; length]; // Always 0
        let mut col3 = vec![Felt::ZERO; length]; // 1 at first row (no transition constraint)
        let mut col4 = vec![Felt::ZERO; length]; // Always 1 (global integrity)
        let mut col5 = vec![Felt::ZERO; length]; // Increments by 1 each row
        let mut col6 = vec![Felt::ZERO; length]; // Always 0 (last row boundary)

        // First row values
        col0[0] = Felt::ONE;
        col1[0] = Felt::ONE;
        col2[0] = Felt::ZERO;
        col3[0] = Felt::ONE;
        col4[0] = Felt::ONE;
        col5[0] = Felt::ZERO;
        col6[0] = Felt::ZERO;

        // Fill subsequent rows based on transition constraints
        for i in 1..length {
            col0[i] = col0[i - 1] + Felt::ONE; // col[0]' = col[0] + 1
            col1[i] = Felt::ZERO; // col[1]' = 0
            col2[i] = col2[i - 1]; // col[2]' = col[2]
            col3[i] = col3[i - 1]; // No constraint, keep same
            col4[i] = Felt::ONE; // Global: col[4] = 1
            col5[i] = col5[i - 1] + Felt::ONE; // col[5]' = col[5] + 1
            col6[i] = Felt::ZERO; // Last row should be 0
        }

        vec![col0, col1, col2, col3, col4, col5, col6]
    }
}

impl CrossBackendTestConfig for ConstantsTestConfig {
    type WinterfellAir = WinterfellConstantsAir;
    type Plonky3Air = Plonky3ConstantsAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        7
    }

    fn trace_length(&self) -> usize {
        self.trace_length
    }

    fn build_winterfell_trace(&self) -> Vec<Vec<Felt>> {
        self.build_trace()
    }

    fn build_winterfell_public_inputs(&self) -> PublicInputs {
        // The Constants AIR has 32 public inputs but doesn't use them in constraints
        // (it uses literal constants like Felt::ONE instead)
        PublicInputs::new([Felt::ZERO; 4], [Felt::ZERO; 4], [Felt::ZERO; 4], [Felt::ZERO; 20])
    }

    fn build_plonky3_public_inputs(&self) -> Vec<Goldilocks> {
        vec![Goldilocks::ZERO; 32]
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellConstantsAir {
        WinterfellConstantsAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3ConstantsAir {
        Plonky3ConstantsAir
    }

    fn num_public_values(&self) -> usize {
        32
    }
}

#[test]
fn test_constants_air_constraint_comparison() {
    let config = ConstantsTestConfig::new(64);
    let result = run_cross_backend_comparison(&config);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "Constants AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_constants_air_constraint_comparison_longer_trace() {
    // Test with a longer trace to exercise more rows
    let config = ConstantsTestConfig::new(128);
    let result = run_cross_backend_comparison(&config);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "Constants AIR comparison (128 rows) passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_constants_air_constraint_comparison_random_inputs() {
    let config = ConstantsTestConfig::new(64);

    // Test with iterations 0 through 50 for thorough coverage
    for iteration in 0u64..=50 {
        let result = run_cross_backend_comparison_random(
            &config,
            "test_constants_air_constraint_comparison_random_inputs",
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

    println!("Constants AIR random comparison passed for all 51 iterations (0-50)");
}
