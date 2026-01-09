//! Cross-backend comparison test for the Fibonacci AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the Fibonacci AIR at every row of the trace.
//!
//! The Fibonacci AIR computes the Fibonacci sequence with constraints:
//! - Boundary: `a.first = stack_inputs[0]`, `b.first = stack_inputs[1]`, `b.last = stack_output[0]`
//! - Transition: `b' = a + b`, `a' = b`

use p3_field::PrimeCharacteristicRing;
use p3_goldilocks::Goldilocks;
use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{
        CrossBackendTestConfig, run_cross_backend_comparison, run_cross_backend_comparison_random,
    },
    tests::fibonacci::{
        fibonacci::{FibonacciAir as WinterfellFibonacciAir, PublicInputs},
        fibonacci_plonky3::FibonacciAir as Plonky3FibonacciAir,
    },
};

// ============================================================================
// Test Configuration
// ============================================================================

/// Configuration for Fibonacci AIR cross-backend comparison tests.
struct FibonacciTestConfig {
    /// The first Fibonacci number (fib_0).
    fib_0: u64,
    /// The second Fibonacci number (fib_1).
    fib_1: u64,
    /// The trace length.
    trace_length: usize,
}

impl FibonacciTestConfig {
    fn new(fib_0: u64, fib_1: u64, trace_length: usize) -> Self {
        Self { fib_0, fib_1, trace_length }
    }

    /// Build a trace for the Fibonacci AIR.
    ///
    /// The Fibonacci AIR has 2 columns (a, b) with the recurrence:
    /// - `a' = b` (next row's a equals current row's b)
    /// - `b' = a + b` (next row's b equals sum of current row's a and b)
    ///
    /// Starting with `a[0] = fib_0`, `b[0] = fib_1`:
    /// ```text
    /// Row | a        | b
    /// ----|----------|----------
    /// 0   | fib_0    | fib_1
    /// 1   | fib_1    | fib_0 + fib_1
    /// 2   | fib_2    | fib_3
    /// ...
    /// ```
    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut col_a = vec![Felt::ZERO; length];
        let mut col_b = vec![Felt::ZERO; length];

        col_a[0] = Felt::new(self.fib_0);
        col_b[0] = Felt::new(self.fib_1);

        for i in 1..length {
            // a' = b (next a is current b)
            col_a[i] = col_b[i - 1];
            // b' = a + b (next b is sum of current a and b)
            col_b[i] = col_a[i - 1] + col_b[i - 1];
        }

        vec![col_a, col_b]
    }

    /// Compute the expected value of `b` at the last step.
    ///
    /// The last step is `trace_length - num_transition_exemptions` where
    /// `num_transition_exemptions = 2` for the Fibonacci AIR.
    fn expected_output(&self) -> Felt {
        let trace = self.build_trace();
        let last_step = self.trace_length - 2; // num_transition_exemptions = 2
        trace[1][last_step] // column b at last_step
    }
}

impl CrossBackendTestConfig for FibonacciTestConfig {
    type WinterfellAir = WinterfellFibonacciAir;
    type Plonky3Air = Plonky3FibonacciAir;
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
        let stack_inputs = [Felt::new(self.fib_0), Felt::new(self.fib_1)];
        let stack_output = [self.expected_output()];
        PublicInputs::new(stack_inputs, stack_output)
    }

    fn build_plonky3_public_inputs(&self) -> Vec<Goldilocks> {
        // Plonky3 public inputs: [stack_inputs[0], stack_inputs[1], stack_output[0]]
        vec![
            Goldilocks::from_u64(self.fib_0),
            Goldilocks::from_u64(self.fib_1),
            Goldilocks::from_u64(self.expected_output().as_int()),
        ]
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellFibonacciAir {
        WinterfellFibonacciAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3FibonacciAir {
        Plonky3FibonacciAir
    }

    fn num_public_values(&self) -> usize {
        3 // stack_inputs[0], stack_inputs[1], stack_output[0]
    }
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_fibonacci_air_constraint_comparison() {
    // Standard Fibonacci starting with 0, 1
    let config = FibonacciTestConfig::new(0, 1, 64);
    let result = run_cross_backend_comparison(&config);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "Fibonacci AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_fibonacci_air_constraint_comparison_different_start() {
    // Fibonacci-like sequence starting with 1, 1
    let config = FibonacciTestConfig::new(1, 1, 64);
    let result = run_cross_backend_comparison(&config);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "Fibonacci AIR comparison (start=1,1) passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_fibonacci_air_constraint_comparison_larger_values() {
    // Test with larger starting values to exercise field arithmetic
    let config = FibonacciTestConfig::new(100, 200, 32);
    let result = run_cross_backend_comparison(&config);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "Fibonacci AIR comparison (start=100,200) passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_fibonacci_air_constraint_comparison_random_inputs() {
    let config = FibonacciTestConfig::new(0, 1, 64);

    // Test with iterations 0 through 50 for thorough coverage
    for iteration in 0u64..=50 {
        let result = run_cross_backend_comparison_random(
            &config,
            "test_fibonacci_air_constraint_comparison_random_inputs",
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

    println!("Fibonacci AIR random comparison passed for all 51 iterations (0-50)");
}
