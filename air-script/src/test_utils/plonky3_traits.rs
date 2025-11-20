use p3_field::{ExtensionField, Field};
use p3_matrix::{
    Matrix,
    dense::{DenseMatrix, RowMajorMatrix, RowMajorMatrixView},
    stack::VerticalPair,
};
use p3_miden_air::{MidenAir, MidenAirBuilder, impl_p3_air_builder_traits};

/// Target trait for AirScript codegen. Implemented by the prover.
pub trait AirScriptBuilder: MidenAirBuilder
where
    <Self as MidenAirBuilder>::F: Field,
{
    /// EF evaluations of periodic columns at the AIR’s random point (z). Order defined by
    /// AirScript.
    fn periodic_evals(&self) -> &[<Self as MidenAirBuilder>::VarEF];

    /// Global challenges in EF. (We can provide defaults; details not important here.)
    fn alpha(&self) -> <Self as MidenAirBuilder>::VarEF;
    fn beta(&self) -> <Self as MidenAirBuilder>::VarEF;
    fn beta_powers(&self) -> &[<Self as MidenAirBuilder>::VarEF];

    /// Aux bus boundary values: EF finals, one per aux/bus column, carried in the proof.
    fn aux_bus_boundary_values(&self) -> &[<Self as MidenAirBuilder>::VarEF];
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
    /// A view of the current and next preprocessed row as a vertical pair.
    preprocessed: VerticalPair<RowMajorMatrixView<'a, F>, RowMajorMatrixView<'a, F>>,
    /// A view of the current and next aux row as a vertical pair.
    aux: Option<VerticalPair<RowMajorMatrixView<'a, EF>, RowMajorMatrixView<'a, EF>>>,
    /// The public values provided for constraint validation (e.g. inputs or outputs).
    public_values: &'a [F],
    /// A flag indicating whether this is the first row.
    is_first_row: F,
    /// A flag indicating whether this is the last row.
    is_last_row: F,
    /// A flag indicating whether this is a transition row (not the last row).
    is_transition: F,
    /// The periodic columns provided for constraint validation.
    periodic_columns: Vec<F>,
    /// The permutation randomness in the extension field.
    permutation_randomness: Vec<EF>,
    /// The aux bus boundary values in the extension field.
    aux_bus_boundary_values: Vec<EF>,
}

impl<'a, F, EF> MidenAirBuilder for DebugConstraintBuilderWithAirScriptTraits<'a, F, EF>
where
    F: Field,
    EF: ExtensionField<F>,
{
    type F = F;
    type Expr = F;
    type Var = F;
    type M = VerticalPair<DenseMatrix<F, &'a [F]>, DenseMatrix<F, &'a [F]>>;
    type PublicVar = F;
    type EF = EF;
    type ExprEF = EF;
    type VarEF = EF;
    type MP = VerticalPair<DenseMatrix<EF, &'a [EF]>, DenseMatrix<EF, &'a [EF]>>;
    type RandomVar = EF;

    fn main(&self) -> Self::M {
        self.main
    }

    fn is_first_row(&self) -> Self::Expr {
        self.is_first_row
    }

    fn is_last_row(&self) -> Self::Expr {
        self.is_last_row
    }

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

    fn public_values(&self) -> &[Self::PublicVar] {
        self.public_values
    }

    fn periodic_evals(&self) -> &[<Self as MidenAirBuilder>::F] {
        self.periodic_columns.as_slice()
    }

    fn preprocessed(&self) -> Self::M {
        self.preprocessed
    }

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

    fn permutation(&self) -> Self::MP {
        self.aux.unwrap()
    }

    fn permutation_randomness(&self) -> &[Self::RandomVar] {
        self.permutation_randomness.as_slice()
    }

    fn aux_bus_boundary_values(&self) -> &[<Self as MidenAirBuilder>::VarEF] {
        self.aux_bus_boundary_values.as_slice()
    }
}

impl_p3_air_builder_traits!(DebugConstraintBuilderWithAirScriptTraits<'a, F, EF> where F: Field, EF: ExtensionField<F>);

pub(crate) fn check_constraints_with_airscript_traits<F, EF, A>(
    air: &A,
    main: &RowMajorMatrix<F>,
    public_values: &Vec<F>,
) where
    F: Field,
    EF: ExtensionField<F>,
    A: MidenAir<F, EF>,
{
    let height = main.height();

    let aux_bus_boundary_values: Vec<_> = (0..air.aux_width()).map(|_| EF::GENERATOR).collect();
    let alpha_f: Vec<F> = (0..2).map(|i| F::from_u64(123456789 * i)).collect(); // Dummy alpha in F
    let alpha = EF::from_basis_coefficients_iter(alpha_f.iter().cloned()).unwrap();
    let beta = EF::from_u64(987654321);
    let beta_powers: Vec<EF> = (0..(air.num_randomness().saturating_sub(1)))
        .map(|power| beta.exp_u64(power as u64))
        .collect();
    let mut permutation_randomness = Vec::with_capacity(air.num_randomness());
    permutation_randomness.push(alpha);
    permutation_randomness.extend_from_slice(beta_powers.as_slice());

    let aux_trace = air.build_aux_trace::<DebugConstraintBuilderWithAirScriptTraits<'_, F, EF>>(
        main,
        &permutation_randomness,
    );

    (0..height).for_each(|i| {
        let i_next = (i + 1) % height;

        let main_local = main.row_slice(i).unwrap(); // i < height so unwrap should never fail.
        let main_next = main.row_slice(i_next).unwrap(); // i_next < height so unwrap should never fail.
        let main = VerticalPair::new(
            RowMajorMatrixView::new_row(&*main_local),
            RowMajorMatrixView::new_row(&*main_next),
        );

        let periodic_columns: Vec<_> =
            air.periodic_table().iter().map(|col| col[i % col.len()]).collect();

        let mut builder = DebugConstraintBuilderWithAirScriptTraits {
            row_index: i,
            main,
            preprocessed: main,
            aux: None,
            public_values,
            is_first_row: F::from_bool(i == 0),
            is_last_row: F::from_bool(i == height - 1),
            is_transition: F::from_bool(i != height - 1),
            periodic_columns,
            permutation_randomness: permutation_randomness.clone(),
            aux_bus_boundary_values: aux_bus_boundary_values.clone(),
        };

        if let Some(aux_trace) = &aux_trace {
            let aux_local = aux_trace.row_slice(i).unwrap(); // i < height so unwrap should never fail.
            let aux_next = aux_trace.row_slice(i_next).unwrap(); // i_next < height so unwrap should never fail.
            let aux = Some(VerticalPair::new(
                RowMajorMatrixView::new_row(&*aux_local),
                RowMajorMatrixView::new_row(&*aux_next),
            ));
            builder.aux = aux;
            air.eval(&mut builder);
        } else {
            air.eval(&mut builder);
        }
    });
}
