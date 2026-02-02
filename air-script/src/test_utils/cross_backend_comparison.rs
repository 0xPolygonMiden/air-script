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

use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
};

use p3_field::{Field, PrimeCharacteristicRing, PrimeField64};
use p3_goldilocks::Goldilocks;
use p3_matrix::{Matrix, dense::RowMajorMatrix};
use p3_miden_air::{MidenAir, MidenAirBuilder};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use winter_air::{
    Air, AuxRandElements, BatchingMethod, EvaluationFrame, FieldExtension,
    ProofOptions as WinterProofOptions, TraceInfo,
};
use winter_math::{FieldElement, ToElements, fields::f64::BaseElement as WinterfellFelt};
use winter_utils::Serializable;

/// The Goldilocks field modulus: 2^64 - 2^32 + 1
const GOLDILOCKS_MODULUS: u64 = 0xFFFF_FFFF_0000_0001;

/// Generates a deterministic seed from a test name and iteration number.
///
/// This ensures different tests get different random traces even with the same iteration,
/// making test results reproducible while avoiding trace collisions across tests.
///
/// # Examples
/// ```ignore
/// let seed = generate_test_seed("my_test", 0);
/// let rng = ChaCha8Rng::seed_from_u64(seed);
/// ```
pub fn generate_test_seed(test_name: &str, iteration: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    test_name.hash(&mut hasher);
    iteration.hash(&mut hasher);
    hasher.finish()
}

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

/// Specifies the source of trace data for cross-backend comparison.
///
/// # Examples
/// ```ignore
/// let result = run_comparison(&config, TraceSource::Default);
/// ```
///
/// ```ignore
/// let result = run_comparison(
///     &config,
///     TraceSource::Random { test_name: "my_test", iteration: 0 },
/// );
/// ```
///
/// ```ignore
/// let trace = vec![vec![WinterfellFelt::ZERO; 64]; 2];
/// let result = run_comparison(&config, TraceSource::Custom(&trace));
/// ```
#[derive(Debug, Clone)]
pub enum TraceSource<'a> {
    /// Use the trace from `config.build_winterfell_trace()`.
    Default,
    /// Generate a random trace with the given seed parameters.
    Random {
        /// Name of the test (used for seed generation).
        test_name: &'static str,
        /// Iteration number (used for seed generation).
        iteration: u64,
    },
    /// Use a custom provided trace.
    Custom(&'a [Vec<WinterfellFelt>]),
}

/// Evaluates Winterfell transition constraints at a specific row,
/// with periodic column values and `is_transition` selector applied.
///
/// This is the unified evaluation function that supports:
/// - Periodic columns (pass values via `periodic_values`)
/// - Transition selectors (applied automatically)
///
/// # Arguments
///
/// * `air` - The Winterfell AIR instance
/// * `trace` - The trace in column-major format
/// * `row` - The row to evaluate at
/// * `num_rows` - Total number of rows in the trace
/// * `periodic_values` - The periodic column values evaluated at this row (empty for simple AIRs)
///
/// # Examples
/// ```ignore
/// let periodic = vec![WinterfellFelt::ONE];
/// let values = evaluate_winterfell_transition(&air, &trace, 0, trace_len, &periodic);
/// ```
pub fn evaluate_winterfell_transition<A>(
    air: &A,
    trace: &[Vec<WinterfellFelt>],
    row: usize,
    num_rows: usize,
    periodic_values: &[WinterfellFelt],
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

    // Allocate result buffer based on main transition constraints only.
    // Winterfell counts main + aux constraints together in the context, but
    // evaluate_transition only writes main constraints.
    let num_constraints = air.context().num_main_transition_constraints();
    let mut result = vec![WinterfellFelt::ZERO; num_constraints];

    // Evaluate transition constraints with periodic values
    air.evaluate_transition(&frame, periodic_values, &mut result);

    // Apply is_transition selector: 1 for rows 0..n-1, 0 for row n-1
    let is_transition = if row < num_rows - 1 {
        WinterfellFelt::ONE
    } else {
        WinterfellFelt::ZERO
    };

    // Convert to canonical u64, applying the selector
    result.iter().map(|e| (is_transition * *e).to_canonical_u64()).collect()
}

/// Evaluates Winterfell auxiliary transition constraints at a specific row.
///
/// This uses the aux trace and verifier randomness, and applies the same
/// is_transition selector as main constraints.
pub fn evaluate_winterfell_aux_transition<A>(
    air: &A,
    main_trace: &[Vec<WinterfellFelt>],
    aux_trace: &[Vec<WinterfellFelt>],
    row: usize,
    num_rows: usize,
    periodic_values: &[WinterfellFelt],
    aux_rand_elements: &AuxRandElements<WinterfellFelt>,
) -> Vec<u64>
where
    A: Air<BaseField = WinterfellFelt>,
{
    let main_width = main_trace.len();
    let aux_width = aux_trace.len();
    let trace_length = main_trace[0].len();

    let main_current: Vec<WinterfellFelt> =
        (0..main_width).map(|col| main_trace[col][row]).collect();
    let next_row = (row + 1) % trace_length;
    let main_next: Vec<WinterfellFelt> =
        (0..main_width).map(|col| main_trace[col][next_row]).collect();
    let main_frame = EvaluationFrame::from_rows(main_current, main_next);

    let aux_current: Vec<WinterfellFelt> = (0..aux_width).map(|col| aux_trace[col][row]).collect();
    let aux_next: Vec<WinterfellFelt> =
        (0..aux_width).map(|col| aux_trace[col][next_row]).collect();
    let aux_frame = EvaluationFrame::from_rows(aux_current, aux_next);

    let num_constraints = air.context().num_aux_transition_constraints();
    let mut result = vec![WinterfellFelt::ZERO; num_constraints];

    air.evaluate_aux_transition(
        &main_frame,
        &aux_frame,
        periodic_values,
        aux_rand_elements,
        &mut result,
    );

    let is_transition = if row < num_rows - 1 {
        WinterfellFelt::ONE
    } else {
        WinterfellFelt::ZERO
    };

    result.iter().map(|e| (is_transition * *e).to_canonical_u64()).collect()
}

/// Evaluates periodic column values at a specific row.
///
/// Periodic columns repeat with a given period. At row `r`, the value is
/// `column[r % period]` where `period` is the length of the column.
///
/// # Arguments
///
/// * `periodic_columns` - Vec of periodic column definitions, each as Vec<u64>
/// * `row` - The row index to evaluate at
///
/// # Returns
///
/// A vector of field elements, one for each periodic column, evaluated at the given row.
///
/// # Examples
/// ```ignore
/// let columns = vec![vec![1, 0, 0, 0], vec![1, 1, 1, 0]];
/// let values: Vec<Goldilocks> = evaluate_periodic_values_at_row(&columns, 5);
/// ```
pub fn evaluate_periodic_values_at_row<F: Field + PrimeCharacteristicRing>(
    periodic_columns: &[Vec<u64>],
    row: usize,
) -> Vec<F> {
    periodic_columns
        .iter()
        .map(|column| {
            if column.is_empty() {
                F::ZERO
            } else {
                let period = column.len();
                let idx = row % period;
                F::from_u64(column[idx])
            }
        })
        .collect()
}

/// Evaluates periodic column values at a specific row for Winterfell (WinterfellFelt).
///
/// # Examples
/// ```ignore
/// let columns = vec![vec![1, 0, 0, 0]];
/// let values = evaluate_winterfell_periodic_at_row(&columns, 2);
/// ```
pub fn evaluate_winterfell_periodic_at_row(
    periodic_columns: &[Vec<u64>],
    row: usize,
) -> Vec<WinterfellFelt> {
    periodic_columns
        .iter()
        .map(|column| {
            if column.is_empty() {
                WinterfellFelt::ZERO
            } else {
                let period = column.len();
                let idx = row % period;
                WinterfellFelt::new(column[idx])
            }
        })
        .collect()
}

/// Gets Winterfell boundary constraint info.
/// Returns (column, row, expected_value) for each assertion.
///
/// # Examples
/// ```ignore
/// let assertions = get_winterfell_boundary_assertions(&air);
/// for (col, row, expected) in assertions {
///     println!("col={col}, row={row}, expected={expected}");
/// }
/// ```
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

/// Gets Winterfell aux boundary constraint info.
/// Returns (column, row, expected_value) for each aux assertion.
pub fn get_winterfell_aux_boundary_assertions<A>(
    air: &A,
    aux_rand_elements: &AuxRandElements<WinterfellFelt>,
) -> Vec<(usize, usize, u64)>
where
    A: Air<BaseField = WinterfellFelt>,
{
    air.get_aux_assertions(aux_rand_elements)
        .iter()
        .map(|assertion| {
            let col = assertion.column();
            let step = assertion.first_step();
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
    /// View of current and next aux rows (if any)
    aux_view: Option<TwoRowMatrixView<F>>,
    /// Current row index being evaluated
    current_row: usize,
    /// Total number of rows in the trace
    num_rows: usize,
    /// Public input values
    public_values: Vec<F>,
    /// Periodic column evaluations (empty for simple AIRs)
    periodic_values: Vec<F>,
    /// Verifier randomness for aux constraints
    randomness: Vec<F>,
    /// Aux bus boundary values
    aux_bus_boundary_values: Vec<F>,
    /// Captured constraint evaluations
    captured_constraints: Vec<F>,
}

impl<F: Field + Clone> ConstraintCapturingBuilder<F> {
    /// Creates a new constraint capturing builder for a specific row.
    pub fn new(
        trace: &RowMajorMatrix<F>,
        aux_trace: Option<&RowMajorMatrix<F>>,
        row: usize,
        public_values: Vec<F>,
        periodic_values: Vec<F>,
        randomness: Vec<F>,
        aux_bus_boundary_values: Vec<F>,
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

        let aux_view = aux_trace.map(|aux| {
            let aux_width = aux.width();
            let aux_current: Vec<F> = aux
                .row_slice(row)
                .map(|s| s.iter().cloned().collect())
                .unwrap_or_else(|| vec![F::ZERO; aux_width]);

            let aux_next_idx = (row + 1) % num_rows;
            let aux_next: Vec<F> = aux
                .row_slice(aux_next_idx)
                .map(|s| s.iter().cloned().collect())
                .unwrap_or_else(|| vec![F::ZERO; aux_width]);

            TwoRowMatrixView::new(aux_current, aux_next)
        });

        Self {
            main_view,
            aux_view,
            current_row: row,
            num_rows,
            public_values,
            periodic_values,
            randomness,
            aux_bus_boundary_values,
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
        match &self.aux_view {
            Some(view) => TwoRowMatrixView::new(view.current_row.clone(), view.next_row.clone()),
            None => {
                // Plonky3 expects a permutation matrix even when aux width is 0.
                // We return an empty view to keep the builder generic without
                // special-casing non-aux AIRs in generated code paths.
                TwoRowMatrixView::new(Vec::new(), Vec::new())
            },
        }
    }

    fn permutation_randomness(&self) -> &[Self::RandomVar] {
        &self.randomness
    }

    fn aux_bus_boundary_values(&self) -> &[Self::VarEF] {
        &self.aux_bus_boundary_values
    }
}

/// Converts a Plonky3 RowMajorMatrix to a Winterfell-style column-major trace.
///
/// # Examples
/// ```ignore
/// let trace = RowMajorMatrix::new(vec![Goldilocks::ZERO; 8], 2);
/// let winterfell_trace = plonky3_trace_to_winterfell(&trace);
/// ```
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
///
/// # Examples
/// ```ignore
/// let trace = vec![vec![WinterfellFelt::ZERO; 4]; 2];
/// let plonky3_trace: RowMajorMatrix<Goldilocks> = winterfell_trace_to_plonky3(&trace);
/// ```
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

/// Compares constraint evaluations row by row.
///
/// Returns a ComparisonResult with any mismatches found.
///
/// # Examples
/// ```ignore
/// let winterfell = vec![vec![0u64, 1u64]];
/// let plonky3 = vec![vec![0u64, 1u64]];
/// let result = compare_evaluations_by_row(&winterfell, &plonky3);
/// assert!(result.is_ok());
/// ```
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

/// Trait for configuring cross-backend comparison tests.
///
/// Implement this trait for each AIR to enable cross-backend constraint comparison.
/// The trait provides a unified interface for building traces, public inputs,
/// and AIR instances for both Winterfell and Plonky3 backends.
///
/// # Example
///
/// ```ignore
/// struct MyTestConfig;
///
/// impl CrossBackendTestConfig for MyTestConfig {
///     type WinterfellAir = MyWinterfellAir;
///     type Plonky3Air = MyPlonky3Air;
///     type WinterfellPublicInputs = MyPublicInputs;
///
///     fn trace_width(&self) -> usize { 2 }
///     fn trace_length(&self) -> usize { 64 }
///     // ... other methods
/// }
/// ```
pub trait CrossBackendTestConfig {
    /// The Winterfell AIR type.
    type WinterfellAir: Air<BaseField = WinterfellFelt>;

    /// The Plonky3 AIR type.
    type Plonky3Air: MidenAir<Goldilocks, Goldilocks>;

    /// The Winterfell public inputs type.
    type WinterfellPublicInputs: Serializable + ToElements<WinterfellFelt>;

    /// Returns the trace width (number of columns).
    fn trace_width(&self) -> usize;

    /// Returns the trace length (number of rows).
    fn trace_length(&self) -> usize;

    /// Builds the trace in Winterfell format (column-major).
    fn build_winterfell_trace(&self) -> Vec<Vec<WinterfellFelt>>;

    /// Builds the Winterfell public inputs.
    fn build_winterfell_public_inputs(&self) -> Self::WinterfellPublicInputs;

    /// Builds the Plonky3 public inputs.
    ///
    /// Default implementation auto-converts from Winterfell public inputs.
    /// Override this if you need custom conversion logic.
    fn build_plonky3_public_inputs(&self) -> Vec<Goldilocks> {
        self.build_winterfell_public_inputs()
            .to_elements()
            .into_iter()
            .map(|felt| Goldilocks::from_u64(felt.as_int()))
            .collect()
    }

    /// Creates the Winterfell AIR instance.
    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: Self::WinterfellPublicInputs,
        options: WinterProofOptions,
    ) -> Self::WinterfellAir;

    /// Creates the Plonky3 AIR instance.
    fn create_plonky3_air(&self) -> Self::Plonky3Air;

    /// Returns the number of public values for Plonky3.
    ///
    /// Default implementation returns the length of Plonky3 public inputs.
    fn num_public_values(&self) -> usize {
        self.build_plonky3_public_inputs().len()
    }

    /// Returns the periodic column values (empty by default).
    ///
    /// Each inner Vec represents a periodic column, with values that repeat
    /// cyclically. The period is determined by the length of each inner Vec.
    ///
    /// For example, `vec![1, 0, 0, 0]` defines a periodic column with period 4
    /// that has value 1 on rows 0, 4, 8, ... and value 0 elsewhere.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn periodic_column_values(&self) -> Vec<Vec<u64>> {
    ///     vec![
    ///         vec![1, 0, 0, 0, 0, 0, 0, 0], // k0: period 8, value 1 at rows 0, 8, 16, ...
    ///         vec![1, 1, 1, 1, 1, 1, 1, 0], // k1: period 8, value 1 except at rows 7, 15, 23, ...
    ///     ]
    /// }
    /// ```
    fn periodic_column_values(&self) -> Vec<Vec<u64>> {
        vec![]
    }

    /// Builds verifier-supplied randomness for auxiliary constraints.
    ///
    /// Randomness is always generated when `num_randomness > 0`. If the length is 0,
    /// this returns an empty vector.
    fn build_aux_randomness(
        &self,
        seed_name: &str,
        iteration: u64,
        num_randomness: usize,
    ) -> Vec<u64> {
        if num_randomness == 0 {
            return vec![];
        }

        let seed = generate_test_seed(seed_name, iteration);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        (0..num_randomness)
            .map(|_| 1 + rng.random_range(0..(GOLDILOCKS_MODULUS - 1)))
            .collect()
    }

    /// Builds a Plonky3 auxiliary trace. Defaults to None; run_comparison will
    /// supply a zeroed aux trace if aux width > 0 and no aux trace is provided.
    fn build_plonky3_aux_trace(
        &self,
        _plonky3_air: &Self::Plonky3Air,
        _main_trace: &RowMajorMatrix<Goldilocks>,
        _randomness: &[Goldilocks],
    ) -> Option<RowMajorMatrix<Goldilocks>> {
        None
    }

    /// Returns aux bus boundary values (empty by default).
    fn aux_bus_boundary_values(&self) -> Vec<u64> {
        vec![]
    }

    /// Builds a random trace in Winterfell format (column-major) using a seeded RNG.
    ///
    /// The seed is generated from the test name and iteration number using
    /// [`generate_test_seed`], ensuring different tests get unique random traces
    /// even when using the same iteration numbers.
    ///
    /// Default implementation generates uniformly random field elements in
    /// the range `[0, GOLDILOCKS_MODULUS)`. Override this if your AIR has
    /// specific requirements for trace structure.
    ///
    /// # Arguments
    /// * `test_name` - Name of the test (used for seed generation)
    /// * `iteration` - Iteration number (used for seed generation)
    fn build_random_winterfell_trace(
        &self,
        test_name: &str,
        iteration: u64,
    ) -> Vec<Vec<WinterfellFelt>> {
        let seed = generate_test_seed(test_name, iteration);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let width = self.trace_width();
        let length = self.trace_length();

        (0..width)
            .map(|_| {
                (0..length)
                    .map(|_| WinterfellFelt::new(rng.random_range(0..GOLDILOCKS_MODULUS)))
                    .collect()
            })
            .collect()
    }
}

/// Creates default proof options for testing.
///
/// # Examples
/// ```ignore
/// let options = default_proof_options();
/// ```
pub fn default_proof_options() -> WinterProofOptions {
    WinterProofOptions::new(
        32,                     // number of queries
        8,                      // blowup factor
        0,                      // grinding factor
        FieldExtension::None,   // field extension
        8,                      // FRI folding factor
        31,                     // FRI max remainder polynomial degree
        BatchingMethod::Linear, // constraint composition batching
        BatchingMethod::Linear, // DEEP polynomial batching
    )
}

/// Runs a full cross-backend comparison.
///
/// This function:
/// 1. Builds or selects a trace based on `source`
/// 2. Creates both Winterfell and Plonky3 AIR instances
/// 3. Evaluates all constraints at each row for both backends
/// 4. Compares the results and returns a detailed report
///
/// # Arguments
///
/// * `config` - The test configuration implementing `CrossBackendTestConfig`
/// * `source` - Where to get the trace data from
///
/// # Examples
/// ```ignore
/// let result = run_comparison(&config, TraceSource::Default);
/// assert!(result.is_ok());
/// ```
///
/// ```ignore
/// let result = run_comparison(
///     &config,
///     TraceSource::Random {
///         test_name: "my_test",
///         iteration: 0,
///     },
/// );
/// ```
pub fn run_comparison<C>(config: &C, source: TraceSource<'_>) -> ComparisonResult
where
    C: CrossBackendTestConfig,
{
    let (winterfell_trace, seed_name, seed_iteration) = match source {
        TraceSource::Default => (config.build_winterfell_trace(), std::any::type_name::<C>(), 0),
        TraceSource::Random { test_name, iteration } => {
            (config.build_random_winterfell_trace(test_name, iteration), test_name, iteration)
        },
        TraceSource::Custom(trace) => (trace.to_vec(), std::any::type_name::<C>(), 0),
    };

    let trace_length = config.trace_length();

    // Convert to Plonky3 format
    let plonky3_trace: RowMajorMatrix<Goldilocks> = winterfell_trace_to_plonky3(&winterfell_trace);

    // Build public inputs
    let winterfell_pub_inputs = config.build_winterfell_public_inputs();
    let plonky3_pub_inputs = config.build_plonky3_public_inputs();

    // Create Plonky3 AIR first so we can determine aux trace parameters.
    let plonky3_air = config.create_plonky3_air();
    let aux_width = plonky3_air.aux_width();
    let num_randomness = plonky3_air.num_randomness();

    // Create Winterfell AIR with appropriate trace info.
    let trace_info = if aux_width > 0 || num_randomness > 0 {
        TraceInfo::new_multi_segment(
            config.trace_width(),
            aux_width,
            num_randomness,
            trace_length,
            vec![],
        )
    } else {
        TraceInfo::new(config.trace_width(), trace_length)
    };
    let proof_options = default_proof_options();
    let winterfell_air =
        config.create_winterfell_air(trace_info, winterfell_pub_inputs, proof_options);

    // Get the last_step for Winterfell (where last-row boundary constraints apply)
    let last_step = trace_length - winterfell_air.context().num_transition_exemptions();

    // Get periodic column definitions
    let periodic_columns = config.periodic_column_values();

    // Aux trace support is optional; when aux width is 0 there is no aux data.
    let aux_randomness_u64 = config.build_aux_randomness(seed_name, seed_iteration, num_randomness);
    let plonky3_randomness: Vec<Goldilocks> =
        aux_randomness_u64.iter().map(|val| Goldilocks::from_u64(*val)).collect();
    let winterfell_randomness: Vec<WinterfellFelt> =
        aux_randomness_u64.iter().map(|val| WinterfellFelt::new(*val)).collect();
    let aux_rand_elements = AuxRandElements::new(winterfell_randomness);

    let aux_trace_plonky3: Option<RowMajorMatrix<Goldilocks>> = if aux_width > 0 {
        let custom_aux =
            config.build_plonky3_aux_trace(&plonky3_air, &plonky3_trace, &plonky3_randomness);
        custom_aux
            .or_else(|| plonky3_air.build_aux_trace(&plonky3_trace, &plonky3_randomness))
            .or_else(|| {
                // Default to an all-zero aux trace when the width is > 0 but no
                // aux trace is provided. This keeps aux support optional for
                // tests that don't exercise auxiliary constraints.
                Some(RowMajorMatrix::new(
                    vec![Goldilocks::ZERO; trace_length * aux_width],
                    aux_width,
                ))
            })
    } else {
        None
    };
    let aux_trace_winterfell = aux_trace_plonky3.as_ref().map(plonky3_trace_to_winterfell);

    let mut aux_boundary_values = config.aux_bus_boundary_values();
    if aux_width > 0 && aux_boundary_values.is_empty() {
        aux_boundary_values = vec![0; aux_width];
    }
    let plonky3_aux_boundary_values: Vec<Goldilocks> =
        aux_boundary_values.iter().map(|val| Goldilocks::from_u64(*val)).collect();

    // Use Winterfell's assertion count to split Plonky3 boundary vs transition
    // constraints since Plonky3 does not report boundary counts separately.
    let main_boundary_count = winterfell_air.get_assertions().len();

    // Evaluate constraints at each row where transition constraints are enforced.
    // Rows >= last_step have transition exemptions in Winterfell, so we skip them
    // to ensure both backends are compared on rows with the same constraint semantics.
    let mut winterfell_results: Vec<Vec<u64>> = Vec::new();
    let mut plonky3_results: Vec<Vec<u64>> = Vec::new();

    for row in 0..last_step {
        // Evaluate periodic values at this row for both backends
        let winterfell_periodic = evaluate_winterfell_periodic_at_row(&periodic_columns, row);
        let plonky3_periodic: Vec<Goldilocks> =
            evaluate_periodic_values_at_row(&periodic_columns, row);

        // Winterfell: evaluate boundary constraints
        let w_boundary = evaluate_winterfell_boundary(
            &winterfell_air,
            &winterfell_trace,
            row,
            trace_length,
            last_step,
        );

        let w_aux_boundary = match aux_trace_winterfell.as_ref() {
            Some(aux_trace) => evaluate_winterfell_aux_boundary(
                &winterfell_air,
                aux_trace,
                row,
                trace_length,
                last_step,
                &aux_rand_elements,
            ),
            None => Vec::new(),
        };

        // Winterfell: evaluate transition constraints with is_transition selector
        // and periodic values
        let w_transition = evaluate_winterfell_transition(
            &winterfell_air,
            &winterfell_trace,
            row,
            trace_length,
            &winterfell_periodic,
        );

        let w_aux_transition = match aux_trace_winterfell.as_ref() {
            Some(aux_trace) => evaluate_winterfell_aux_transition(
                &winterfell_air,
                &winterfell_trace,
                aux_trace,
                row,
                trace_length,
                &winterfell_periodic,
                &aux_rand_elements,
            ),
            None => Vec::new(),
        };

        // Combine: to follow the order of emitted constraints in Plonky3, we inject:
        // - main boundary constraints
        // - aux boundary constraints (Winterfell-only: Plonky3 eval does not emit explicit aux
        //   boundary assertions; bus boundary values are handled outside eval)
        // - main transition constraints
        // - aux transition constraints
        // TODO: Best guess for now: Plonky3 eval does not emit aux boundary assertions,
        // so we are not comparing those constraints independently across backends.
        // We replicate Winterfell's aux boundary evaluations into the Plonky3 result
        // to keep constraint ordering consistent. Follow-up: emit aux boundary
        // assertions in Plonky3 codegen so both backends produce them directly,
        // then remove this injection.
        let mut w_all = w_boundary;
        w_all.extend(w_aux_boundary.clone());
        w_all.extend(w_transition);
        w_all.extend(w_aux_transition);
        winterfell_results.push(w_all);

        // Plonky3: create a capturing builder for this row with periodic values
        let mut builder = ConstraintCapturingBuilder::new(
            &plonky3_trace,
            aux_trace_plonky3.as_ref(),
            row,
            plonky3_pub_inputs.clone(),
            plonky3_periodic,
            plonky3_randomness.clone(),
            plonky3_aux_boundary_values.clone(),
        );

        // Evaluate the Plonky3 AIR
        MidenAir::<Goldilocks, Goldilocks>::eval(&plonky3_air, &mut builder);

        // Get captured constraints
        let p_all = builder.get_captured_constraints();

        let p_all = if !w_aux_boundary.is_empty() && p_all.len() >= main_boundary_count {
            let (p_boundary, p_transition) = p_all.split_at(main_boundary_count);
            let mut combined = Vec::with_capacity(p_all.len() + w_aux_boundary.len());
            combined.extend_from_slice(p_boundary);
            // Plonky3 generated AIRs do not emit explicit aux boundary assertions,
            // so we evaluate them from the Winterfell AIR and insert them here
            // to compare all constraints in a consistent order.
            combined.extend_from_slice(&w_aux_boundary);
            combined.extend_from_slice(p_transition);
            combined
        } else {
            p_all
        };

        plonky3_results.push(p_all);
    }

    // Compare results
    compare_evaluations_by_row(&winterfell_results, &plonky3_results)
}

/// Evaluates boundary constraints at a specific row for Winterfell.
///
/// This function evaluates boundary constraints with proper handling of the last step
/// from Winterfell's AIR context, which accounts for transition exemptions when
/// determining when last-row boundary constraints should apply.
///
/// # Arguments
/// * `air` - The Winterfell AIR instance
/// * `trace` - The trace in column-major format
/// * `row` - The row to evaluate at
/// * `num_rows` - Total number of rows in the trace
/// * `last_step` - The last step index from AIR context (accounts for transition exemptions)
///
/// # Examples
/// ```ignore
/// let values = evaluate_winterfell_boundary(&air, &trace, 0, trace_len, last_step);
/// ```
pub fn evaluate_winterfell_boundary<A>(
    air: &A,
    trace: &[Vec<WinterfellFelt>],
    row: usize,
    num_rows: usize,
    last_step: usize,
) -> Vec<u64>
where
    A: Air<BaseField = WinterfellFelt>,
{
    let assertions = get_winterfell_boundary_assertions(air);
    let mut results = Vec::new();

    for (col, assertion_row, expected) in assertions {
        let actual = trace[col][row].to_canonical_u64();

        let is_first_row = if row == 0 { 1u64 } else { 0u64 };
        let is_last_row = if row == num_rows - 1 { 1u64 } else { 0u64 };

        if assertion_row == 0 {
            // First row boundary constraint
            let actual_felt = WinterfellFelt::new(actual);
            let expected_felt = WinterfellFelt::new(expected);
            let is_first_felt = WinterfellFelt::new(is_first_row);
            let diff = actual_felt - expected_felt;
            let result = is_first_felt * diff;
            results.push(result.to_canonical_u64());
        } else if assertion_row == last_step {
            // Last row boundary constraint (using last_step from AIR)
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

/// Evaluates auxiliary boundary constraints at a specific row for Winterfell.
pub fn evaluate_winterfell_aux_boundary<A>(
    air: &A,
    aux_trace: &[Vec<WinterfellFelt>],
    row: usize,
    num_rows: usize,
    last_step: usize,
    aux_rand_elements: &AuxRandElements<WinterfellFelt>,
) -> Vec<u64>
where
    A: Air<BaseField = WinterfellFelt>,
{
    let assertions = get_winterfell_aux_boundary_assertions(air, aux_rand_elements);
    let mut results = Vec::new();

    for (col, assertion_row, expected) in assertions {
        let actual = aux_trace[col][row].to_canonical_u64();

        let is_first_row = if row == 0 { 1u64 } else { 0u64 };
        let is_last_row = if row == num_rows - 1 { 1u64 } else { 0u64 };

        if assertion_row == 0 {
            let actual_felt = WinterfellFelt::new(actual);
            let expected_felt = WinterfellFelt::new(expected);
            let is_first_felt = WinterfellFelt::new(is_first_row);
            let diff = actual_felt - expected_felt;
            let result = is_first_felt * diff;
            results.push(result.to_canonical_u64());
        } else if assertion_row == last_step {
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
