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

    /// Number of alpha challenges used in the AIR.
    fn num_alpha_challenges(&self) -> usize {
        0
    }

    /// Periodic constants (base-field) backing periodic_evals().
    // fn periodic_table(&self) -> &'static [&'static [F]];
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
    fn alpha_powers(&self) -> &[<Self as ExtensionBuilder>::VarEF];
    fn beta(&self) -> <Self as ExtensionBuilder>::VarEF;

    /// Aux bus boundary values: EF finals, one per aux/bus column, carried in the proof.
    fn aux_bus_boundary_values(&self) -> &[<Self as ExtensionBuilder>::VarEF];
}

// Helper macros for Plonky3 test generation

/// Generates a Plonky3 AIR test function with the standard boilerplate
///
/// # Arguments
/// * `test_name` - The identifier for the test function (e.g., `test_binary_air`)
/// * `air_name` - The identifier for the AIR struct (e.g., `BinaryAir`)
#[macro_export]
macro_rules! generate_air_plonky3_test_with_airscript_traits {
    ($test_name:ident, $air_name:ident) => {
        #[test]
        fn $test_name() {
            type Val = Goldilocks;
            type Challenge = BinomialExtensionField<Val, 5>;

            type ByteHash = Sha256;
            type FieldHash = SerializingHasher<ByteHash>;
            type MyCompress = CompressionFunctionFromHasher<ByteHash, 2, 32>;
            type ValMmcs = MerkleTreeMmcs<Val, u8, FieldHash, MyCompress, 32>;
            type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
            type Challenger = SerializingChallenger64<Val, HashChallenger<u8, ByteHash, 32>>;
            type Pcs = CirclePcs<Val, ValMmcs, ChallengeMmcs>;
            type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;

            let byte_hash = ByteHash {};
            let field_hash = FieldHash::new(Sha256);
            let compress = MyCompress::new(byte_hash);
            let val_mmcs = ValMmcs::new(field_hash, compress);
            let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
            let challenger = Challenger::from_hasher(vec![], byte_hash);
            let fri_params = create_benchmark_fri_params(challenge_mmcs);
            let pcs = Pcs {
                mmcs: val_mmcs,
                fri_params,
                _phantom: PhantomData,
            };
            let config = MyConfig::new(pcs, challenger);

            let inputs = generate_inputs();
            let inputs_goldilocks: Vec<Val> =
                inputs.iter().map(|&x| Val::from_u32(x)).collect();

            let trace = generate_trace_rows::<Val>(inputs);

            check_constraints_with_airscript_traits::<Goldilocks, Challenge, $air_name>(&$air_name {}, &trace, &inputs_goldilocks);

            /*let prove_with_periodic_columns = prove_with_periodic_columns(&config, &BitwiseAir {}, trace, &inputs_goldilocks);
            verify_with_periodic_columns(&config, &BitwiseAir {}, &prove_with_periodic_columns, &inputs_goldilocks).expect("Verification failed");*/

            /*let proof = prove(&config, &BitwiseAir {}, trace, &inputs_goldilocks);
            verify(&config, &BitwiseAir {}, &proof, &inputs_goldilocks).expect("Verification failed");*/
        }
    };
}

/// A builder that runs constraint assertions during testing.
///
/// Used in conjunction with [`check_constraints`] to simulate
/// an execution trace and verify that the AIR logic enforces all constraints.
#[derive(Debug)]
pub struct DebugConstraintBuilderWithAirScriptTraits<'a, F: Field, EF: ExtensionField<F>> {
    /// The index of the row currently being evaluated.
    row_index: usize,
    /// A view of the current and next row as a vertical pair.
    main: VerticalPair<RowMajorMatrixView<'a, F>, RowMajorMatrixView<'a, F>>,
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
    /// The alpha powers in the extension field.
    alpha_powers: Vec<EF>,
    /// The beta challenge in the extension field.
    beta: EF,
    /// The aux bus boundary values in the extension field.
    aux_bus_boundary_values: Vec<EF>,
    /// The aux trace as a vertical pair.
    permutation: VerticalPair<RowMajorMatrixView<'a, EF>, RowMajorMatrixView<'a, EF>>,
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
        self.permutation
    }

    fn permutation_randomness(&self) -> &[Self::RandomVar] {
        self.alpha_powers.as_slice()
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

    fn alpha_powers(&self) -> &[<Self as ExtensionBuilder>::VarEF] {
        self.alpha_powers.as_slice()
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
    alpha_challenges: Vec<EF>,
    beta: EF,
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
    let multiset_inserts: EF = ((beta
        + alpha_challenges[0]
        + (EF::from_u64(3) + EF::from(main_current[1].clone())) * alpha_challenges[1]
        + EF::from(main_current[0].clone()) * alpha_challenges[2])
        * EF::from(main_current[2].clone())
        + EF::ONE
        - EF::from(main_current[2].clone()))
        * ((beta
            + alpha_challenges[0].double()
            + EF::from(main_current[1].clone()) * alpha_challenges[1])
            * (EF::ONE - EF::from(main_current[2].clone()))
            + EF::from(main_current[2].clone()));
    let multiset_removals: EF = ((beta
        + alpha_challenges[0]
        + (EF::from_u64(3) + EF::from(main_current[1].clone())) * alpha_challenges[1]
        + EF::from(main_current[1].clone()) * alpha_challenges[2])
        * EF::from(main_current[3].clone())
        + EF::ONE
        - EF::from(main_current[3].clone()))
        * ((beta
            + alpha_challenges[0].double()
            + EF::from(main_current[0].clone()) * alpha_challenges[1])
            * (EF::ONE - EF::from(main_current[3].clone()))
            + EF::from(main_current[3].clone()));
    let multiset_current = EF::from(aux_current[0].clone());
    let multiset_next = multiset_current * multiset_inserts * multiset_removals.inverse();

    // Second bus: logup
    // 0 = A * q + B + C - D * q' - E;
    let a: EF = (beta
        + EF::from_u64(3) * alpha_challenges[0]
        + EF::from(main_current[0].clone()) * alpha_challenges[1])
        * (beta
            + EF::from_u64(3) * alpha_challenges[0]
            + EF::from(main_current[0].clone()) * alpha_challenges[1])
        * (beta
            + EF::from_u64(3) * alpha_challenges[0]
            + EF::from(main_current[1].clone()) * alpha_challenges[1]);
    let b: EF = (beta
        + EF::from_u64(3) * alpha_challenges[0]
        + EF::from(main_current[0].clone()) * alpha_challenges[1])
        * (beta
            + EF::from_u64(3) * alpha_challenges[0]
            + EF::from(main_current[1].clone()) * alpha_challenges[1])
        * EF::from(main_current[4].clone());
    let c: EF = (beta
        + EF::from_u64(3) * alpha_challenges[0]
        + EF::from(main_current[0].clone()) * alpha_challenges[1])
        * (beta
            + EF::from_u64(3) * alpha_challenges[0]
            + EF::from(main_current[1].clone()) * alpha_challenges[1])
        * EF::from(main_current[5].clone());
    let d: EF = (beta
        + EF::from_u64(3) * alpha_challenges[0]
        + EF::from(main_current[0].clone()) * alpha_challenges[1])
        * (beta
            + EF::from_u64(3) * alpha_challenges[0]
            + EF::from(main_current[0].clone()) * alpha_challenges[1])
        * (beta
            + EF::from_u64(3) * alpha_challenges[0]
            + EF::from(main_current[1].clone()) * alpha_challenges[1]);
    let e: EF = (beta
        + EF::from_u64(3) * alpha_challenges[0]
        + EF::from(main_current[0].clone()) * alpha_challenges[1])
        * (beta
            + EF::from_u64(3) * alpha_challenges[0]
            + EF::from(main_current[0].clone()) * alpha_challenges[1])
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
    let alpha = EF::from_u64(123456789);
    let beta = EF::from_u64(987654321);
    let alpha_powers: Vec<EF> = (0..air.num_alpha_challenges())
        .map(|power| alpha.exp_u64(power as u64))
        .collect();

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
                compute_aux_transition::<F, EF>(main, alpha_powers.clone(), beta, aux_local);
        }
        let aux_next = current_aux_values;
        let aux = VerticalPair::new(
            RowMajorMatrixView::new_row(&aux_local),
            RowMajorMatrixView::new_row(&aux_next),
        );

        let mut builder = DebugConstraintBuilderWithAirScriptTraits {
            row_index: i,
            main,
            public_values,
            is_first_row: F::from_bool(i == 0),
            is_last_row: F::from_bool(i == height - 1),
            is_transition: F::from_bool(i != height - 1),
            periodic_columns,
            alpha,
            beta,
            alpha_powers: alpha_powers.clone(),
            aux_bus_boundary_values: aux_bus_boundary_values.clone(),
            permutation: aux,
        };

        AirScriptAir::eval(air, &mut builder);
    });
}
