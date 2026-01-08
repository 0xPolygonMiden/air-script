//! Cross-backend constraint evaluation comparison utilities.
//!
//! This module provides infrastructure for comparing constraint evaluations between
//! Winterfell and Plonky3 backends. It verifies that both backends produce equivalent
//! constraint evaluation results for the same trace data.
//!
//! # Design
//!
//! The comparison works by:
//! 1. Building the same trace data for both backends
//! 2. Evaluating all constraints at each row using both backends
//! 3. Comparing the canonical u64 representations of the results
//!
//! For Plonky3, we use a `ConstraintCapturingBuilder` that implements `MidenAirBuilder`
//! and captures all values passed to `assert_zero` calls.
//!
//! For Winterfell, we directly call `evaluate_transition` and manually evaluate boundary
//! constraints to produce comparable results.

use std::fmt;

use p3_field::{Field, PrimeCharacteristicRing, PrimeField64};
use p3_goldilocks::Goldilocks;
use p3_matrix::{Matrix, dense::RowMajorMatrix};
use p3_miden_air::MidenAirBuilder;
use winter_air::{Air, EvaluationFrame};
use winter_math::{FieldElement, fields::f64::BaseElement as WinterfellFelt};

// ============================================================================
// Canonical u64 Conversion
// ============================================================================

/// Trait for converting field elements to their canonical u64 representation.
///
/// This is used for comparing field elements across different backends that may
/// use different internal representations (e.g., Montgomery form vs raw).
pub trait ToCanonicalU64 {
    fn to_canonical_u64(&self) -> u64;
}

impl ToCanonicalU64 for WinterfellFelt {
    fn to_canonical_u64(&self) -> u64 {
        self.as_int()
    }
}

impl ToCanonicalU64 for Goldilocks {
    fn to_canonical_u64(&self) -> u64 {
        self.as_canonical_u64()
    }
}

// ============================================================================
// Constraint Mismatch Reporting
// ============================================================================

/// Represents a single constraint evaluation mismatch between backends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintMismatch {
    pub row: usize,
    pub constraint_index: usize,
    pub winterfell_value: u64,
    pub plonky3_value: u64,
}

impl fmt::Display for ConstraintMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Row {}, Constraint {}: Winterfell={}, Plonky3={}",
            self.row, self.constraint_index, self.winterfell_value, self.plonky3_value
        )
    }
}

/// Result of comparing constraint evaluations between backends.
#[derive(Debug)]
pub struct ComparisonResult {
    pub mismatches: Vec<ConstraintMismatch>,
    pub total_rows: usize,
    pub total_constraints_checked: usize,
}

impl ComparisonResult {
    pub fn is_ok(&self) -> bool {
        self.mismatches.is_empty()
    }

    /// Formats the comparison result for display.
    pub fn format_report(&self) -> String {
        if self.is_ok() {
            return format!(
                "All constraints match! Checked {} constraints across {} rows.",
                self.total_constraints_checked, self.total_rows
            );
        }

        let mut report = String::new();
        report.push_str("Constraint evaluation mismatches found!\n\n");

        for mismatch in &self.mismatches {
            report.push_str(&format!("{}\n", mismatch));
        }

        report.push_str(&format!(
            "\nSummary: {} mismatches found across {} rows ({} total constraints checked)",
            self.mismatches.len(),
            self.total_rows,
            self.total_constraints_checked
        ));

        report
    }
}

// ============================================================================
// Winterfell Constraint Evaluation
// ============================================================================

/// Evaluates Winterfell transition constraints at a specific row.
///
/// Returns a vector of constraint evaluation values as canonical u64.
pub fn evaluate_winterfell_transition_at_row<A>(
    air: &A,
    trace: &[Vec<WinterfellFelt>],
    row: usize,
) -> Vec<u64>
where
    A: Air<BaseField = WinterfellFelt>,
{
    let trace_width = trace.len();
    let trace_length = trace[0].len();

    // Build current and next row data
    let current: Vec<WinterfellFelt> = (0..trace_width).map(|col| trace[col][row]).collect();

    let next_row = (row + 1) % trace_length;
    let next: Vec<WinterfellFelt> = (0..trace_width).map(|col| trace[col][next_row]).collect();

    // Create evaluation frame
    let frame = EvaluationFrame::from_rows(current, next);

    // Allocate result buffer based on number of transition constraints
    let num_constraints = air.context().num_transition_constraints();
    let mut result = vec![WinterfellFelt::ZERO; num_constraints];

    // Evaluate transition constraints (empty periodic values for simple AIRs)
    let periodic_values: Vec<WinterfellFelt> = vec![];
    air.evaluate_transition(&frame, &periodic_values, &mut result);

    // Convert to canonical u64
    result.iter().map(|e| e.to_canonical_u64()).collect()
}

/// Gets Winterfell boundary constraint info.
/// Returns (column, row, expected_value) for each assertion.
pub fn get_winterfell_boundary_assertions<A>(air: &A) -> Vec<(usize, usize, u64)>
where
    A: Air<BaseField = WinterfellFelt>,
{
    air.get_assertions()
        .iter()
        .map(|assertion| {
            // For single assertions, get the value directly
            // The assertion contains: column index, step (row), and expected value
            let col = assertion.column();
            let step = assertion.first_step();
            // Get values from the assertion - for single value assertions
            let values = assertion.values();
            let expected = if !values.is_empty() {
                values[0].to_canonical_u64()
            } else {
                0
            };
            (col, step, expected)
        })
        .collect()
}

/// Evaluates boundary constraints at a specific row for Winterfell.
/// Returns the constraint evaluation (actual - expected) for each boundary constraint
/// that applies to this row, multiplied by the first_row indicator (like Plonky3 does).
pub fn evaluate_winterfell_boundary_at_row<A>(
    air: &A,
    trace: &[Vec<WinterfellFelt>],
    row: usize,
    num_rows: usize,
) -> Vec<u64>
where
    A: Air<BaseField = WinterfellFelt>,
{
    let assertions = get_winterfell_boundary_assertions(air);
    let mut results = Vec::new();

    for (col, assertion_row, expected) in assertions {
        // Compute (actual - expected)
        let actual = trace[col][row].to_canonical_u64();

        // For first row constraints: multiply by is_first_row indicator
        // For last row constraints: multiply by is_last_row indicator
        let is_first_row = if row == 0 { 1u64 } else { 0u64 };
        let is_last_row = if row == num_rows - 1 { 1u64 } else { 0u64 };

        if assertion_row == 0 {
            // First row boundary constraint
            // Plonky3 computes: is_first_row * (actual - expected)
            // We need to do the same arithmetic in the field
            let actual_felt = WinterfellFelt::new(actual);
            let expected_felt = WinterfellFelt::new(expected);
            let is_first_felt = WinterfellFelt::new(is_first_row);
            let diff = actual_felt - expected_felt;
            let result = is_first_felt * diff;
            results.push(result.to_canonical_u64());
        } else if assertion_row == num_rows - 1 {
            // Last row boundary constraint
            let actual_felt = WinterfellFelt::new(actual);
            let expected_felt = WinterfellFelt::new(expected);
            let is_last_felt = WinterfellFelt::new(is_last_row);
            let diff = actual_felt - expected_felt;
            let result = is_last_felt * diff;
            results.push(result.to_canonical_u64());
        }
    }

    results
}

// ============================================================================
// Plonky3 Constraint Capturing Builder
// ============================================================================

/// A view into two consecutive rows of the trace matrix for constraint evaluation.
pub struct TwoRowMatrixView<F> {
    current_row: Vec<F>,
    next_row: Vec<F>,
}

impl<F: Clone> TwoRowMatrixView<F> {
    pub fn new(current_row: Vec<F>, next_row: Vec<F>) -> Self {
        Self { current_row, next_row }
    }
}

impl<F: Clone + Send + Sync> Matrix<F> for TwoRowMatrixView<F> {
    fn width(&self) -> usize {
        self.current_row.len()
    }

    fn height(&self) -> usize {
        2
    }

    fn row_slice(&self, r: usize) -> Option<impl core::ops::Deref<Target = [F]>> {
        match r {
            0 => Some(self.current_row.clone()),
            1 => Some(self.next_row.clone()),
            _ => None,
        }
    }
}

/// A builder that captures constraint evaluation values instead of asserting them.
///
/// This implements `MidenAirBuilder` and records all values passed to `assert_zero`
/// for later comparison with Winterfell's constraint evaluations.
pub struct ConstraintCapturingBuilder<F: Field> {
    /// View of current and next rows
    main_view: TwoRowMatrixView<F>,
    /// Current row index being evaluated
    current_row: usize,
    /// Total number of rows in the trace
    num_rows: usize,
    /// Public input values
    public_values: Vec<F>,
    /// Periodic column evaluations (empty for simple AIRs)
    periodic_values: Vec<F>,
    /// Captured constraint evaluations
    captured_constraints: Vec<F>,
}

impl<F: Field + Clone> ConstraintCapturingBuilder<F> {
    /// Creates a new constraint capturing builder for a specific row.
    pub fn new(
        trace: &RowMajorMatrix<F>,
        row: usize,
        public_values: Vec<F>,
        periodic_values: Vec<F>,
    ) -> Self {
        let num_rows = trace.height();
        let width = trace.width();

        // Get current row
        let current_row: Vec<F> = trace
            .row_slice(row)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_else(|| vec![F::ZERO; width]);

        // Get next row (wrap around)
        let next_row_idx = (row + 1) % num_rows;
        let next_row: Vec<F> = trace
            .row_slice(next_row_idx)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_else(|| vec![F::ZERO; width]);

        let main_view = TwoRowMatrixView::new(current_row, next_row);

        Self {
            main_view,
            current_row: row,
            num_rows,
            public_values,
            periodic_values,
            captured_constraints: Vec::new(),
        }
    }

    /// Returns the captured constraint values as canonical u64.
    pub fn get_captured_constraints(&self) -> Vec<u64>
    where
        F: PrimeField64,
    {
        self.captured_constraints.iter().map(|f| f.as_canonical_u64()).collect()
    }

    fn is_first_row_value(&self) -> F {
        if self.current_row == 0 { F::ONE } else { F::ZERO }
    }

    fn is_last_row_value(&self) -> F {
        if self.current_row == self.num_rows - 1 {
            F::ONE
        } else {
            F::ZERO
        }
    }

    fn is_transition_window_value(&self, size: usize) -> F {
        if self.current_row < self.num_rows.saturating_sub(size - 1) {
            F::ONE
        } else {
            F::ZERO
        }
    }
}

impl<F: Field + PrimeCharacteristicRing + Clone> MidenAirBuilder for ConstraintCapturingBuilder<F> {
    type F = F;
    type Expr = F;
    type Var = F;
    type M = TwoRowMatrixView<F>;
    type PublicVar = F;
    type PeriodicVal = F;
    type EF = F;
    type ExprEF = F;
    type VarEF = F;
    type MP = TwoRowMatrixView<F>;
    type RandomVar = F;

    fn main(&self) -> Self::M {
        TwoRowMatrixView::new(self.main_view.current_row.clone(), self.main_view.next_row.clone())
    }

    fn is_first_row(&self) -> Self::Expr {
        self.is_first_row_value()
    }

    fn is_last_row(&self) -> Self::Expr {
        self.is_last_row_value()
    }

    fn is_transition_window(&self, size: usize) -> Self::Expr {
        self.is_transition_window_value(size)
    }

    fn assert_zero<I: Into<Self::Expr>>(&mut self, x: I) {
        self.captured_constraints.push(x.into());
    }

    fn public_values(&self) -> &[Self::PublicVar] {
        &self.public_values
    }

    fn periodic_evals(&self) -> &[Self::PeriodicVal] {
        &self.periodic_values
    }

    fn preprocessed(&self) -> Self::M {
        self.main()
    }

    fn assert_zero_ext<I>(&mut self, x: I)
    where
        I: Into<Self::ExprEF>,
    {
        self.captured_constraints.push(x.into());
    }

    fn permutation(&self) -> Self::MP {
        self.main()
    }

    fn permutation_randomness(&self) -> &[Self::RandomVar] {
        &[]
    }

    fn aux_bus_boundary_values(&self) -> &[Self::VarEF] {
        &[]
    }
}

// ============================================================================
// Trace Conversion Utilities
// ============================================================================

/// Converts a Plonky3 RowMajorMatrix to a Winterfell-style column-major trace.
pub fn plonky3_trace_to_winterfell<F>(trace: &RowMajorMatrix<F>) -> Vec<Vec<WinterfellFelt>>
where
    F: PrimeField64 + Clone + Send + Sync,
{
    let num_rows = trace.height();
    let num_cols = trace.width();

    (0..num_cols)
        .map(|col| {
            (0..num_rows)
                .map(|row| {
                    let row_slice = trace.row_slice(row).unwrap();
                    WinterfellFelt::new(row_slice[col].as_canonical_u64())
                })
                .collect()
        })
        .collect()
}

/// Converts a Winterfell column-major trace to a Plonky3 RowMajorMatrix.
pub fn winterfell_trace_to_plonky3<F: Field + PrimeCharacteristicRing>(
    trace: &[Vec<WinterfellFelt>],
) -> RowMajorMatrix<F> {
    if trace.is_empty() {
        return RowMajorMatrix::new(vec![], 0);
    }

    let num_cols = trace.len();
    let num_rows = trace[0].len();

    let mut values = Vec::with_capacity(num_rows * num_cols);

    for row in 0..num_rows {
        for col in 0..num_cols {
            let val = trace[col][row].as_int();
            values.push(F::from_u64(val));
        }
    }

    RowMajorMatrix::new(values, num_cols)
}

// ============================================================================
// High-Level Comparison Functions
// ============================================================================

/// Compares constraint evaluations row by row.
///
/// Returns a ComparisonResult with any mismatches found.
pub fn compare_evaluations_by_row(
    winterfell_evals: &[Vec<u64>],
    plonky3_evals: &[Vec<u64>],
) -> ComparisonResult {
    let total_rows = winterfell_evals.len().max(plonky3_evals.len());
    let mut mismatches = Vec::new();
    let mut total_constraints_checked = 0;

    for row in 0..total_rows {
        let w_row = winterfell_evals.get(row);
        let p_row = plonky3_evals.get(row);

        match (w_row, p_row) {
            (Some(w), Some(p)) => {
                // Check if constraint counts match
                if w.len() != p.len() {
                    mismatches.push(ConstraintMismatch {
                        row,
                        constraint_index: usize::MAX,
                        winterfell_value: w.len() as u64,
                        plonky3_value: p.len() as u64,
                    });
                    continue;
                }

                total_constraints_checked += w.len();

                for (idx, (w_val, p_val)) in w.iter().zip(p.iter()).enumerate() {
                    if w_val != p_val {
                        mismatches.push(ConstraintMismatch {
                            row,
                            constraint_index: idx,
                            winterfell_value: *w_val,
                            plonky3_value: *p_val,
                        });
                    }
                }
            },
            (Some(w), None) => {
                mismatches.push(ConstraintMismatch {
                    row,
                    constraint_index: usize::MAX,
                    winterfell_value: w.len() as u64,
                    plonky3_value: 0,
                });
            },
            (None, Some(p)) => {
                mismatches.push(ConstraintMismatch {
                    row,
                    constraint_index: usize::MAX,
                    winterfell_value: 0,
                    plonky3_value: p.len() as u64,
                });
            },
            (None, None) => {},
        }
    }

    ComparisonResult {
        mismatches,
        total_rows,
        total_constraints_checked,
    }
}
