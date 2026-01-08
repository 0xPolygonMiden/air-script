//! Cross-backend comparison test for the Binary AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the Binary AIR at every row of the trace.

use p3_field::PrimeCharacteristicRing;
use p3_goldilocks::Goldilocks;
use p3_matrix::dense::RowMajorMatrix;
use p3_miden_air::MidenAir;
use pretty_assertions::assert_eq;
use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{
        ConstraintCapturingBuilder, compare_evaluations_by_row,
        evaluate_winterfell_boundary_at_row, evaluate_winterfell_transition_at_row,
        winterfell_trace_to_plonky3,
    },
    tests::binary::{
        binary::{BinaryAir as WinterfellBinaryAir, PublicInputs},
        binary_plonky3::BinaryAir as Plonky3BinaryAir,
    },
};

/// Build a trace for the Binary AIR.
///
/// The Binary AIR has 2 columns (a, b) that alternate between 0 and 1:
/// - Row 0: (start, start)
/// - Row 1: (1-start, 1-start)
/// - Row 2: (start, start)
/// - ...
fn build_binary_trace(length: usize, start_value: u64) -> Vec<Vec<Felt>> {
    let mut col_a = vec![Felt::ZERO; length];
    let mut col_b = vec![Felt::ZERO; length];

    let start = Felt::new(start_value);
    let one = Felt::ONE;

    col_a[0] = start;
    col_b[0] = start;

    for i in 1..length {
        col_a[i] = one - col_a[i - 1];
        col_b[i] = one - col_b[i - 1];
    }

    vec![col_a, col_b]
}

/// Build public inputs for the Binary AIR.
fn build_public_inputs(first_value: u64) -> ([Felt; 16], Vec<Goldilocks>) {
    let mut winterfell_inputs = [Felt::ZERO; 16];
    winterfell_inputs[0] = Felt::new(first_value);

    let plonky3_inputs: Vec<Goldilocks> = (0..16)
        .map(|i| {
            if i == 0 {
                Goldilocks::from_u64(first_value)
            } else {
                Goldilocks::ZERO
            }
        })
        .collect();

    (winterfell_inputs, plonky3_inputs)
}

#[test]
fn test_binary_air_constraint_comparison() {
    // Test parameters
    let trace_length = 64; // Small trace for testing
    let start_value = 0u64; // Start with 0 (satisfies binary constraint a*a - a = 0)

    // Build trace (Winterfell format: column-major)
    let winterfell_trace = build_binary_trace(trace_length, start_value);

    // Convert to Plonky3 format (row-major)
    let plonky3_trace: RowMajorMatrix<Goldilocks> = winterfell_trace_to_plonky3(&winterfell_trace);

    // Build public inputs
    let (winterfell_pub_inputs, plonky3_pub_inputs) = build_public_inputs(start_value);

    // Create Winterfell AIR
    let trace_info = TraceInfo::new(2, trace_length);
    let proof_options = WinterProofOptions::new(
        32,
        8,
        0,
        winter_air::FieldExtension::None,
        8,
        31,
        winter_air::BatchingMethod::Linear,
        winter_air::BatchingMethod::Linear,
    );
    let winterfell_air = WinterfellBinaryAir::new(
        trace_info,
        PublicInputs::new(winterfell_pub_inputs),
        proof_options,
    );

    // Create Plonky3 AIR
    let plonky3_air = Plonky3BinaryAir;

    // Evaluate constraints at each row and collect results
    let mut winterfell_results: Vec<Vec<u64>> = Vec::new();
    let mut plonky3_results: Vec<Vec<u64>> = Vec::new();

    for row in 0..trace_length {
        // Winterfell: evaluate transition constraints
        let w_transition =
            evaluate_winterfell_transition_at_row(&winterfell_air, &winterfell_trace, row);

        // Winterfell: evaluate boundary constraints (with selector multiplication)
        let w_boundary = evaluate_winterfell_boundary_at_row(
            &winterfell_air,
            &winterfell_trace,
            row,
            trace_length,
        );

        // Combine: boundary constraints first, then transition constraints
        // This matches the order in the Plonky3 generated code
        let mut w_all = w_boundary;
        w_all.extend(w_transition);
        winterfell_results.push(w_all);

        // Plonky3: create a capturing builder for this row
        let mut builder = ConstraintCapturingBuilder::new(
            &plonky3_trace,
            row,
            plonky3_pub_inputs.clone(),
            vec![], // No periodic values for binary AIR
        );

        // Evaluate the Plonky3 AIR
        MidenAir::<Goldilocks, Goldilocks>::eval(&plonky3_air, &mut builder);

        // Get captured constraints
        let p_all = builder.get_captured_constraints();
        plonky3_results.push(p_all);
    }

    // Compare results
    let comparison = compare_evaluations_by_row(&winterfell_results, &plonky3_results);

    if !comparison.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", comparison.format_report());
    }

    // Also use pretty_assertions for a nice diff if there are mismatches
    assert_eq!(
        winterfell_results, plonky3_results,
        "Winterfell and Plonky3 constraint evaluations should match"
    );

    println!(
        "Binary AIR comparison passed: {} constraints checked across {} rows",
        comparison.total_constraints_checked, comparison.total_rows
    );
}

/// Test with a different starting value to ensure constraints are properly evaluated.
#[test]
fn test_binary_air_constraint_comparison_start_one() {
    let trace_length = 64;
    let start_value = 1u64; // Start with 1

    let winterfell_trace = build_binary_trace(trace_length, start_value);
    let plonky3_trace: RowMajorMatrix<Goldilocks> = winterfell_trace_to_plonky3(&winterfell_trace);

    let (winterfell_pub_inputs, plonky3_pub_inputs) = build_public_inputs(start_value);

    let trace_info = TraceInfo::new(2, trace_length);
    let proof_options = WinterProofOptions::new(
        32,
        8,
        0,
        winter_air::FieldExtension::None,
        8,
        31,
        winter_air::BatchingMethod::Linear,
        winter_air::BatchingMethod::Linear,
    );
    let winterfell_air = WinterfellBinaryAir::new(
        trace_info,
        PublicInputs::new(winterfell_pub_inputs),
        proof_options,
    );

    let plonky3_air = Plonky3BinaryAir;

    let mut winterfell_results: Vec<Vec<u64>> = Vec::new();
    let mut plonky3_results: Vec<Vec<u64>> = Vec::new();

    for row in 0..trace_length {
        let w_transition =
            evaluate_winterfell_transition_at_row(&winterfell_air, &winterfell_trace, row);
        let w_boundary = evaluate_winterfell_boundary_at_row(
            &winterfell_air,
            &winterfell_trace,
            row,
            trace_length,
        );

        let mut w_all = w_boundary;
        w_all.extend(w_transition);
        winterfell_results.push(w_all);

        let mut builder = ConstraintCapturingBuilder::new(
            &plonky3_trace,
            row,
            plonky3_pub_inputs.clone(),
            vec![],
        );

        MidenAir::<Goldilocks, Goldilocks>::eval(&plonky3_air, &mut builder);
        plonky3_results.push(builder.get_captured_constraints());
    }

    let comparison = compare_evaluations_by_row(&winterfell_results, &plonky3_results);

    if !comparison.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", comparison.format_report());
    }

    assert_eq!(winterfell_results, plonky3_results);
}
