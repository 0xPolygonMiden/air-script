use p3_air::{
    Air, AirBuilder, AirBuilderWithPublicValues, ExtensionBuilder, PermutationAirBuilder,
};
use p3_field::{ExtensionField, Field};
use p3_matrix::{
    Matrix,
    dense::{RowMajorMatrix, RowMajorMatrixView},
    stack::VerticalPair,
};

/// Miden/AirScript-specific AIR. Replaces BaseAir for this forked target.
pub trait AirScriptAir<F: Field, AB: AirScriptBuilder<F = F>> {
    /// Auxiliary width of the AIR.
    fn aux_width(&self) -> usize {
        0
    }

    /// Maximum number of beta challenge powers needed.
    /// This number corresponds to the largest tuple of Field elements
    /// that are inserted into/removed from a bus.
    fn num_beta_challenges(&self) -> usize {
        0
    }

    /// Periodic constants (base-field) backing periodic_evals().
    fn periodic_table(&self) -> Vec<Vec<F>>;

    /// Single entrypoint: encodes main + aux + boundary constraints.
    fn eval(&self, builder: &mut AB);
}

/// Target trait for AirScript codegen. Implemented by the prover.
pub trait AirScriptBuilder:
    AirBuilder + AirBuilderWithPublicValues + ExtensionBuilder + PermutationAirBuilder
where
    <Self as AirBuilder>::F: Field,
{
    /// EF evaluations of periodic columns at the AIR’s random point (z). Order defined by
    /// AirScript.
    fn periodic_evals(&self) -> &[<Self as ExtensionBuilder>::VarEF];

    /// Global challenges in EF. (We can provide defaults; details not important here.)
    fn alpha(&self) -> <Self as ExtensionBuilder>::VarEF;
    fn beta(&self) -> <Self as ExtensionBuilder>::VarEF;
    fn beta_powers(&self) -> &[<Self as ExtensionBuilder>::VarEF];

    /// Aux bus boundary values: EF finals, one per aux/bus column, carried in the proof.
    fn aux_bus_boundary_values(&self) -> &[<Self as ExtensionBuilder>::VarEF];
}

/// A builder that runs constraint assertions during testing.
///
/// Used in conjunction with [`check_constraints`] to simulate
/// an execution trace and verify that the AIR logic enforces all constraints.
#[derive(Debug)]
pub struct DebugConstraintBuilderWithAirScriptTraits<'a, F: Field, EF: ExtensionField<F>> {
    /// The index of the row currently being evaluated.
    row_index: usize,
    /// A view of the current and next main row as a vertical pair.
    main: VerticalPair<RowMajorMatrixView<'a, F>, RowMajorMatrixView<'a, F>>,
    /// A view of the current and next aux row as a vertical pair.
    aux: VerticalPair<RowMajorMatrixView<'a, EF>, RowMajorMatrixView<'a, EF>>,
    /// The public values provided for constraint validation (e.g. inputs or outputs).
    public_values: &'a [F],
    /// A flag indicating whether this is the first row.
    is_first_row: F,
    /// A flag indicating whether this is the last row.
    is_last_row: F,
    /// A flag indicating whether this is a transition row (not the last row).
    is_transition: F,
    /// The periodic columns provided for constraint validation.
    periodic_columns: Vec<EF>,
    /// The alpha challenge in the extension field.
    alpha: EF,
    /// The beta challenge in the extension field.
    beta: EF,
    /// The beta powers in the extension field.
    beta_powers: Vec<EF>,
    /// The permutation randomness in the extension field.
    permutation_randomness: Vec<EF>,
    /// The aux bus boundary values in the extension field.
    aux_bus_boundary_values: Vec<EF>,
}

impl<'a, F, EF> AirBuilder for DebugConstraintBuilderWithAirScriptTraits<'a, F, EF>
where
    F: Field,
    EF: ExtensionField<F>,
{
    type F = F;
    type Expr = F;
    type Var = F;
    type M = VerticalPair<RowMajorMatrixView<'a, F>, RowMajorMatrixView<'a, F>>;

    fn main(&self) -> Self::M {
        self.main
    }

    fn is_first_row(&self) -> Self::Expr {
        self.is_first_row
    }

    fn is_last_row(&self) -> Self::Expr {
        self.is_last_row
    }

    /// # Panics
    /// This function panics if `size` is not `2`.
    fn is_transition_window(&self, size: usize) -> Self::Expr {
        if size == 2 {
            self.is_transition
        } else {
            panic!("only supports a window size of 2")
        }
    }

    fn assert_zero<I: Into<Self::Expr>>(&mut self, x: I) {
        assert_eq!(x.into(), F::ZERO, "constraints had nonzero value on row {}", self.row_index);
    }

    fn assert_eq<I1: Into<Self::Expr>, I2: Into<Self::Expr>>(&mut self, x: I1, y: I2) {
        let x = x.into();
        let y = y.into();
        assert_eq!(x, y, "values didn't match on row {}: {} != {}", self.row_index, x, y);
    }
}

impl<'a, F, EF> AirBuilderWithPublicValues for DebugConstraintBuilderWithAirScriptTraits<'a, F, EF>
where
    F: Field,
    EF: ExtensionField<F>,
{
    type PublicVar = Self::F;

    fn public_values(&self) -> &[Self::F] {
        self.public_values
    }
}

impl<'a, F, EF> ExtensionBuilder for DebugConstraintBuilderWithAirScriptTraits<'a, F, EF>
where
    F: Field,
    EF: ExtensionField<F>,
{
    type EF = EF;

    type ExprEF = EF;

    type VarEF = EF;

    fn assert_zero_ext<I>(&mut self, x: I)
    where
        I: Into<Self::ExprEF>,
    {
        assert_eq!(
            x.into(),
            EF::ZERO,
            "constraints on ext field had nonzero value on row {}",
            self.row_index
        );
    }
}

impl<'a, F, EF> PermutationAirBuilder for DebugConstraintBuilderWithAirScriptTraits<'a, F, EF>
where
    F: Field,
    EF: ExtensionField<F>,
{
    type MP = VerticalPair<RowMajorMatrixView<'a, EF>, RowMajorMatrixView<'a, EF>>;

    type RandomVar = EF;

    fn permutation(&self) -> Self::MP {
        self.aux
    }

    fn permutation_randomness(&self) -> &[Self::RandomVar] {
        self.permutation_randomness.as_slice()
    }
}

impl<'a, F, EF> AirScriptBuilder for DebugConstraintBuilderWithAirScriptTraits<'a, F, EF>
where
    F: Field + Into<Self::Expr>,
    EF: ExtensionField<F>,
{
    fn periodic_evals(&self) -> &[<Self as ExtensionBuilder>::VarEF] {
        self.periodic_columns.as_slice()
    }

    fn alpha(&self) -> <Self as ExtensionBuilder>::VarEF {
        self.alpha
    }

    fn beta_powers(&self) -> &[<Self as ExtensionBuilder>::VarEF] {
        self.beta_powers.as_slice()
    }

    fn beta(&self) -> <Self as ExtensionBuilder>::VarEF {
        self.beta
    }

    fn aux_bus_boundary_values(&self) -> &[<Self as ExtensionBuilder>::VarEF] {
        self.aux_bus_boundary_values.as_slice()
    }
}

fn compute_aux_transition<F, EF>(
    main: VerticalPair<RowMajorMatrixView<F>, RowMajorMatrixView<F>>,
    alpha: EF,
    beta_challenges: Vec<EF>,
    aux_current: [EF; 2],
) -> [EF; 2]
where
    F: Field,
    EF: ExtensionField<F>,
{
    let main_current = &main.row_slice(0).unwrap();
    let _main_next = &main.row_slice(1).unwrap();

    // First bus: multiset
    // p' * multiset_removals = p * multiset_inserts
    let multiset_inserts: EF = ((alpha
        + beta_challenges[0]
        + (EF::from_u64(3) + EF::from(main_current[1].clone())) * beta_challenges[1]
        + EF::from(main_current[0].clone()) * beta_challenges[2])
        * EF::from(main_current[2].clone())
        + EF::ONE
        - EF::from(main_current[2].clone()))
        * ((alpha
            + beta_challenges[0].double()
            + EF::from(main_current[1].clone()) * beta_challenges[1])
            * (EF::ONE - EF::from(main_current[2].clone()))
            + EF::from(main_current[2].clone()));
    let multiset_removals: EF = ((alpha
        + beta_challenges[0]
        + (EF::from_u64(3) + EF::from(main_current[1].clone())) * beta_challenges[1]
        + EF::from(main_current[1].clone()) * beta_challenges[2])
        * EF::from(main_current[3].clone())
        + EF::ONE
        - EF::from(main_current[3].clone()))
        * ((alpha
            + beta_challenges[0].double()
            + EF::from(main_current[0].clone()) * beta_challenges[1])
            * (EF::ONE - EF::from(main_current[3].clone()))
            + EF::from(main_current[3].clone()));
    let multiset_current = EF::from(aux_current[0].clone());
    let multiset_next = multiset_current * multiset_inserts * multiset_removals.inverse();

    // Second bus: logup
    // 0 = A * q + B + C - D * q' - E;
    let a: EF = (alpha
        + EF::from_u64(3) * beta_challenges[0]
        + EF::from(main_current[0].clone()) * beta_challenges[1])
        * (alpha
            + EF::from_u64(3) * beta_challenges[0]
            + EF::from(main_current[0].clone()) * beta_challenges[1])
        * (alpha
            + EF::from_u64(3) * beta_challenges[0]
            + EF::from(main_current[1].clone()) * beta_challenges[1]);
    let b: EF = (alpha
        + EF::from_u64(3) * beta_challenges[0]
        + EF::from(main_current[0].clone()) * beta_challenges[1])
        * (alpha
            + EF::from_u64(3) * beta_challenges[0]
            + EF::from(main_current[1].clone()) * beta_challenges[1])
        * EF::from(main_current[4].clone());
    let c: EF = (alpha
        + EF::from_u64(3) * beta_challenges[0]
        + EF::from(main_current[0].clone()) * beta_challenges[1])
        * (alpha
            + EF::from_u64(3) * beta_challenges[0]
            + EF::from(main_current[1].clone()) * beta_challenges[1])
        * EF::from(main_current[5].clone());
    let d: EF = (alpha
        + EF::from_u64(3) * beta_challenges[0]
        + EF::from(main_current[0].clone()) * beta_challenges[1])
        * (alpha
            + EF::from_u64(3) * beta_challenges[0]
            + EF::from(main_current[0].clone()) * beta_challenges[1])
        * (alpha
            + EF::from_u64(3) * beta_challenges[0]
            + EF::from(main_current[1].clone()) * beta_challenges[1]);
    let e: EF = (alpha
        + EF::from_u64(3) * beta_challenges[0]
        + EF::from(main_current[0].clone()) * beta_challenges[1])
        * (alpha
            + EF::from_u64(3) * beta_challenges[0]
            + EF::from(main_current[0].clone()) * beta_challenges[1])
        * EF::from(main_current[6].clone());
    let logup_current = EF::from(aux_current[1].clone());
    let logup_next = (a * logup_current + b + c - e) * d.inverse();

    // Dummy implementation for illustration purposes.
    [multiset_next, logup_next]
}

pub(crate) fn check_constraints_with_airscript_traits<F, EF, A>(
    air: &A,
    main: &RowMajorMatrix<F>,
    public_values: &Vec<F>,
) where
    F: Field,
    EF: ExtensionField<F>,
    A: for<'a> Air<DebugConstraintBuilderWithAirScriptTraits<'a, F, EF>>,
    A: for<'a> AirScriptAir<F, DebugConstraintBuilderWithAirScriptTraits<'a, F, EF>>,
{
    let height = main.height();

    let aux_bus_boundary_values: Vec<_> = (0..air.aux_width()).map(|_| EF::GENERATOR).collect();
    let alpha_f: Vec<F> = (0..2).map(|i| F::from_u64(123456789 * i)).collect(); // Dummy alpha in F
    let alpha = EF::from_basis_coefficients_iter(alpha_f.iter().cloned()).unwrap();
    let beta = EF::from_u64(987654321);
    let beta_powers: Vec<EF> = (0..air.num_beta_challenges())
        .map(|power| alpha.exp_u64(power as u64))
        .collect();
    let mut permutation_randomness = Vec::with_capacity(1 + beta_powers.len());
    permutation_randomness.push(alpha);
    permutation_randomness.extend_from_slice(beta_powers.as_slice());

    let initial_aux = [EF::ONE, EF::ZERO];

    let mut current_aux_values = initial_aux.clone();

    (0..height).for_each(|i| {
        let i_next = (i + 1) % height;

        let main_local = main.row_slice(i).unwrap(); // i < height so unwrap should never fail.
        let main_next = main.row_slice(i_next).unwrap(); // i_next < height so unwrap should never fail.
        let main = VerticalPair::new(
            RowMajorMatrixView::new_row(&*main_local),
            RowMajorMatrixView::new_row(&*main_next),
        );

        let periodic_columns_base: Vec<_> =
            air.periodic_table().iter().map(|col| col[i % col.len()]).collect();
        let periodic_columns: Vec<EF> =
            periodic_columns_base.iter().map(|&v| EF::from(v)).collect();

        let aux_local = current_aux_values;
        if air.aux_width() > 0 && i != height - 1 {
            current_aux_values =
                compute_aux_transition::<F, EF>(main, alpha, beta_powers.clone(), aux_local);
        }
        let aux_next = current_aux_values;
        let aux = VerticalPair::new(
            RowMajorMatrixView::new_row(&aux_local),
            RowMajorMatrixView::new_row(&aux_next),
        );

        let mut builder = DebugConstraintBuilderWithAirScriptTraits {
            row_index: i,
            main,
            aux,
            public_values,
            is_first_row: F::from_bool(i == 0),
            is_last_row: F::from_bool(i == height - 1),
            is_transition: F::from_bool(i != height - 1),
            periodic_columns,
            alpha,
            beta,
            beta_powers: beta_powers.clone(),
            permutation_randomness: permutation_randomness.clone(),
            aux_bus_boundary_values: aux_bus_boundary_values.clone(),
        };

        AirScriptAir::eval(air, &mut builder);
    });
}
