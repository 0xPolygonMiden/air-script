use p3_field::{ExtensionField, Field, PrimeCharacteristicRing};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrixView;
use p3_matrix::stack::VerticalPair;
use p3_miden_air::{MidenAir, MidenAirBuilder, RowMajorMatrix};

pub const MAIN_WIDTH: usize = 1;
pub const AUX_WIDTH: usize = 2;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 2;
pub const MAX_BETA_CHALLENGE_POWER: usize = 2;

pub struct BusesAir;

impl<F, EF> MidenAir<F, EF> for BusesAir
where F: Field,
      EF: ExtensionField<F>,
{
    fn width(&self) -> usize {
        MAIN_WIDTH
    }

    fn num_randomness(&self) -> usize {
        1 + MAX_BETA_CHALLENGE_POWER
    }

    fn aux_width(&self) -> usize {
        AUX_WIDTH
    }

    fn build_aux_trace(&self, _main: &RowMajorMatrix<F>, _challenges: &[EF]) -> Option<RowMajorMatrix<EF>> {
        // Note: consider using Some(build_aux_trace_with_miden_vm::<F, EF>(_main, _challenges, module)) if you want to build the aux trace using Miden VM aux trace builders.

        let num_rows = _main.height();
        let trace_length = num_rows * AUX_WIDTH;
        let mut long_trace = EF::zero_vec(trace_length);
        let mut trace = RowMajorMatrix::new(long_trace, AUX_WIDTH);
        let (prefix, rows, suffix) = unsafe { trace.values.align_to_mut::<[EF; AUX_WIDTH]>() };
        assert!(prefix.is_empty(), "Alignment should match");
        assert!(suffix.is_empty(), "Alignment should match");
        assert_eq!(rows.len(), num_rows);
        // Initialize first row
        let initial_values = Self::buses_initial_values::<F, EF>();
        for j in 0..AUX_WIDTH {
            rows[0][j] = initial_values[j];
        }
        // Fill subsequent rows using direct access to the rows array
        for i in 0..num_rows-1 {
            let i_next = (i + 1) % num_rows;
            let main_local = _main.row_slice(i).unwrap(); // i < height so unwrap should never fail.
            let main_next = _main.row_slice(i_next).unwrap(); // i_next < height so unwrap should never fail.
            let main = VerticalPair::new(
                RowMajorMatrixView::new_row(&*main_local),
                RowMajorMatrixView::new_row(&*main_next),
            );
            let periodic_values: [_; NUM_PERIODIC_VALUES] = <BusesAir as MidenAir<F, EF>>::periodic_table(self).iter().map(|col| col[i % col.len()]).collect::<Vec<_>>().try_into().expect("Wrong number of periodic values");
            let prev_row = &rows[i];
            let next_row = Self::buses_transitions::<F, EF>(
                &main,
                _challenges,
                &periodic_values,
                prev_row,
            );
            for j in 0..AUX_WIDTH {
                rows[i+1][j] = next_row[j];
            }
        }
        Some(trace)
    }

    fn eval<AB>(&self, builder: &mut AB)
    where AB: MidenAirBuilder<F = F, EF = EF>,
    {
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values: [_; NUM_PERIODIC_VALUES] = builder.periodic_evals().try_into().expect("Wrong number of periodic values");
        let preprocessed = builder.preprocessed();
        let main = builder.main();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        let (&alpha, beta_challenges) = builder.permutation_randomness().split_first().expect("Wrong number of randomness");
        let beta_challenges: [_; MAX_BETA_CHALLENGE_POWER] = beta_challenges.try_into().expect("Wrong number of randomness");
        let aux_bus_boundary_values: [_; AUX_WIDTH] = builder.aux_bus_boundary_values().try_into().expect("Wrong number of aux bus boundary values");
        let aux = builder.permutation();
        let (aux_current, aux_next) = (
            aux.row_slice(0).unwrap(),
            aux.row_slice(1).unwrap(),
        );

        // Main boundary constraints

        // Main integrity/transition constraints

        // Aux boundary constraints
        builder.when_last_row().assert_zero_ext(AB::ExprEF::from(aux_current[0].clone().into()) - AB::ExprEF::ONE);
        builder.when_first_row().assert_zero_ext(AB::ExprEF::from(aux_current[1].clone().into()));
        builder.when_last_row().assert_zero_ext(AB::ExprEF::from(aux_current[1].clone().into()));

        // Aux integrity/transition constraints
        builder.when_transition().assert_zero_ext(((alpha.into() + beta_challenges[0].into()) * AB::ExprEF::from(main_current[0].clone().into()) + AB::ExprEF::ONE - AB::ExprEF::from(main_current[0].clone().into())) * AB::ExprEF::from(aux_current[0].clone().into()) - ((alpha.into() + beta_challenges[0].into()) * (AB::ExprEF::ONE - AB::ExprEF::from(main_current[0].clone().into())) + AB::ExprEF::from(main_current[0].clone().into())) * AB::ExprEF::from(aux_next[0].clone().into()));
        builder.when_transition().assert_zero_ext((alpha.into() + beta_challenges[0].into() + beta_challenges[1].into().double()) * (alpha.into() + beta_challenges[0].into() + beta_challenges[1].into().double()) * AB::ExprEF::from(aux_current[1].clone().into()) + (alpha.into() + beta_challenges[0].into() + beta_challenges[1].into().double()) * AB::ExprEF::from(main_current[0].clone().into()) - ((alpha.into() + beta_challenges[0].into() + beta_challenges[1].into().double()) * (alpha.into() + beta_challenges[0].into() + beta_challenges[1].into().double()) * AB::ExprEF::from(aux_next[1].clone().into()) + (alpha.into() + beta_challenges[0].into() + beta_challenges[1].into().double()).double()));
    }
}

impl BusesAir {
    fn buses_initial_values<F, EF>() -> Vec<EF>
    where F: Field,
          EF: ExtensionField<F>,
    {
        vec![
            EF::ZERO,
        ]
    }

    fn buses_transitions<F, EF>(main: &VerticalPair<RowMajorMatrixView<F>, RowMajorMatrixView<F>>, challenges: &[EF], periodic_evals: &[F], aux_current: &[EF]) -> Vec<EF>
    where F: Field,
          EF: ExtensionField<F>,
    {
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        let (&alpha, beta_challenges) = challenges.split_first().expect("Wrong number of randomness");
        let beta_challenges: [_; MAX_BETA_CHALLENGE_POWER] = beta_challenges.try_into().expect("Wrong number of randomness");
        let periodic_values: [_; NUM_PERIODIC_VALUES] = periodic_evals.try_into().expect("Wrong number of periodic values");
        vec![
            (((alpha + beta_challenges[0]) * EF::from(main_current[0].clone()) + EF::ONE - EF::from(main_current[0].clone())) * EF::from(aux_current[0].clone())) * ((alpha + beta_challenges[0]) * (EF::ONE - EF::from(main_current[0].clone())) + EF::from(main_current[0].clone())).inverse(),
            ((alpha + beta_challenges[0] + beta_challenges[1].double()) * (alpha + beta_challenges[0] + beta_challenges[1].double()) * EF::from(aux_current[1].clone()) + (alpha + beta_challenges[0] + beta_challenges[1].double()) * EF::from(main_current[0].clone()) - (alpha + beta_challenges[0] + beta_challenges[1].double()).double()) * ((alpha + beta_challenges[0] + beta_challenges[1].double()) * (alpha + beta_challenges[0] + beta_challenges[1].double())).inverse(),
        ]
    }
}