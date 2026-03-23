use p3_field::{ExtensionField, Field, PrimeCharacteristicRing};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrixView;
use p3_matrix::stack::VerticalPair;
use p3_miden_air::{BusType, MidenAir, MidenAirBuilder, RowMajorMatrix};
use miden_processor::utils::uninit_vector;

pub const MAIN_WIDTH: usize = 1;
pub const AUX_WIDTH: usize = 1;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 2;
pub const MAX_BETA_CHALLENGE_POWER: usize = 1;

pub struct BusesSumFormAir;

impl<F, EF> MidenAir<F, EF> for BusesSumFormAir
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

    fn bus_types(&self) -> Vec<BusType> {
        vec![
            BusType::Multiset,
        ]
    }

    fn build_aux_trace(&self, _main: &RowMajorMatrix<F>, _challenges: &[EF]) -> Option<RowMajorMatrix<F>> {
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
        let multiset_indices: Vec<usize> = vec![0];
        let logup_indices: Vec<usize> = vec![];

        // Multiset columns: same pattern as Miden AuxColumnBuilder::build_aux_column (request/response, batch inversion).
        // Fill multiset column 0
        let mut requests: Vec<EF> = unsafe { uninit_vector(num_rows) };
        requests[0] = EF::ONE;
        let mut responses_prod: Vec<EF> = unsafe { uninit_vector(num_rows) };
        responses_prod[0] = EF::ONE;
        let mut requests_running_prod = requests[0];
        // Product of all requests to be inverted, used to compute inverses of requests. (Miden utils.rs build_aux_column)
        for i in 0..num_rows - 1 {
            let i_next = (i + 1) % num_rows;
            let main_local = _main.row_slice(i).unwrap(); // i < height so unwrap should never fail.
            let main_next = _main.row_slice(i_next).unwrap(); // i_next < height so unwrap should never fail.
            let main = VerticalPair::new(
                RowMajorMatrixView::new_row(&*main_local),
                RowMajorMatrixView::new_row(&*main_next),
            );
            let periodic_values: [_; NUM_PERIODIC_VALUES] = <BusesSumFormAir as MidenAir<F, EF>>::periodic_table(self).iter().map(|col| col[i % col.len()]).collect::<Vec<_>>().try_into().expect("Wrong number of periodic values");

            let response = Self::bus_0_multiset_responses_at::<F, EF>(&main, _challenges, &periodic_values);
            responses_prod[i + 1] = responses_prod[i] * response;
            let request = Self::bus_0_multiset_requests_at::<F, EF>(&main, _challenges, &periodic_values);
            requests[i + 1] = request;
            requests_running_prod *= request;
        }

        // Use batch-inversion method to compute running product of `response[i]/request[i]`.
        for i in 0..num_rows {
            rows[i][0] = responses_prod[i];
        }
        let mut requests_running_divisor = requests_running_prod.inverse();
        for i in (0..num_rows).rev() {
            rows[i][0] *= requests_running_divisor;
            requests_running_divisor *= requests[i];
        }

        // Logup columns: denominator/product over factors, and per-row term contributing
        // to the running sum (LogUp). Batch-invert denominators then build running sum.
        let n = num_rows - 1;
        let trace_f = trace.flatten_to_base();
        Some(trace_f)
    }

    fn eval<AB>(&self, builder: &mut AB)
    where AB: MidenAirBuilder<F = F>,
    {
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values: [_; NUM_PERIODIC_VALUES] = builder.periodic_evals().try_into().expect("Wrong number of periodic values");
        // Note: for now, we do not have any preprocessed values
        // let preprocessed = builder.preprocessed();
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
        builder.when_first_row().assert_zero(main_current[0].clone().into());

        // Main integrity/transition constraints
        builder.assert_zero(main_current[0].clone().into() * main_current[0].clone().into() - main_current[0].clone().into());

        // Aux integrity/transition constraints
        builder.when_transition().assert_zero_ext(((alpha.into() + beta_challenges[0].into()) * (AB::ExprEF::ONE - AB::ExprEF::from(main_current[0].clone().into())) + AB::ExprEF::from(main_current[0].clone().into())) * AB::ExprEF::from(aux_next[0].clone().into()) - ((alpha.into() + beta_challenges[0].into()) * AB::ExprEF::from(main_current[0].clone().into()) + AB::ExprEF::ONE - AB::ExprEF::from(main_current[0].clone().into())) * AB::ExprEF::from(aux_current[0].clone().into()));
    }
}

impl BusesSumFormAir {
    fn buses_initial_values<F, EF>() -> Vec<EF>
    where F: Field,
          EF: ExtensionField<F>,
    {
        vec![
            EF::ONE,
        ]
    }

    fn bus_0_multiset_requests_at<F, EF>(main: &VerticalPair<RowMajorMatrixView<F>, RowMajorMatrixView<F>>, challenges: &[EF], periodic_evals: &[F]) -> EF
    where F: Field,
          EF: ExtensionField<F>,
    {
        let (main_current, _main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        let _periodic_values: [_; NUM_PERIODIC_VALUES] = periodic_evals.try_into().expect("Wrong number of periodic values");
        let result = (challenges[0] + challenges[1] * EF::ONE) * (EF::ONE - EF::from(main_current[0].clone())) + (EF::ONE - (EF::ONE - EF::from(main_current[0].clone())));
        if result == EF::ZERO {
            return EF::ONE;
        }
        result
    }

    fn bus_0_multiset_responses_at<F, EF>(main: &VerticalPair<RowMajorMatrixView<F>, RowMajorMatrixView<F>>, challenges: &[EF], periodic_evals: &[F]) -> EF
    where F: Field,
          EF: ExtensionField<F>,
    {
        let (main_current, _main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        let _periodic_values: [_; NUM_PERIODIC_VALUES] = periodic_evals.try_into().expect("Wrong number of periodic values");
        let result = (challenges[0] + challenges[1] * EF::ONE) * EF::from(main_current[0].clone()) + (EF::ONE - EF::from(main_current[0].clone()));
        if result == EF::ZERO {
            return EF::ONE;
        }
        result
    }
}