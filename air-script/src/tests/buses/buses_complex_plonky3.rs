use p3_field::{ExtensionField, Field, PrimeCharacteristicRing};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrixView;
use p3_miden_air::{MidenAir, MidenAirBuilder, RowMajorMatrix};

pub const MAIN_WIDTH: usize = 7;
pub const AUX_WIDTH: usize = 2;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 2;
pub const NUM_BETA_CHALLENGES: usize = 3;

pub struct BusesAir;

impl<F, EF> MidenAir<F, EF> for BusesAir
where F: Field,
      EF: ExtensionField<F>,
{
    fn width(&self) -> usize {
        MAIN_WIDTH
    }

    fn num_randomness(&self) -> usize {
        1 + NUM_BETA_CHALLENGES
    }

    fn aux_width(&self) -> usize {
        AUX_WIDTH
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
        let (&alpha, beta_challenges) = builder.permutation_randomness().split_first().unwrap();
        let beta_challenges: [_; NUM_BETA_CHALLENGES] = beta_challenges.try_into().expect("Wrong number of randomness");
        let aux_bus_boundary_values: [_; AUX_WIDTH] = builder.aux_bus_boundary_values().try_into().expect("Wrong number of aux bus boundary values");
        let aux = builder.permutation();
        let (aux_current, aux_next) = (
            aux.row_slice(0).unwrap(),
            aux.row_slice(1).unwrap(),
        );

        // Main boundary constraints
        builder.when_first_row().assert_zero(main_current[0].clone().into());

        // Main integrity/transition constraints
        builder.assert_zero(main_current[2].clone().into() * main_current[2].clone().into() - main_current[2].clone().into());
        builder.assert_zero(main_current[3].clone().into() * main_current[3].clone().into() - main_current[3].clone().into());

        // Aux boundary constraints
        builder.when_first_row().assert_zero_ext(AB::ExprEF::from(aux_current[0].clone().into()) - AB::ExprEF::ONE);
        builder.when_last_row().assert_zero_ext(AB::ExprEF::from(aux_current[0].clone().into()) - AB::ExprEF::ONE);
        builder.when_first_row().assert_zero_ext(AB::ExprEF::from(aux_current[1].clone().into()));
        builder.when_last_row().assert_zero_ext(AB::ExprEF::from(aux_current[1].clone().into()));

        // Aux integrity/transition constraints
        builder.when_transition().assert_zero_ext(((alpha.into() + beta_challenges[0].into() + (AB::ExprEF::from_u64(3) + AB::ExprEF::from(main_current[1].clone().into())) * beta_challenges[1].into() + AB::ExprEF::from(main_current[0].clone().into()) * beta_challenges[2].into()) * AB::ExprEF::from(main_current[2].clone().into()) + AB::ExprEF::ONE - AB::ExprEF::from(main_current[2].clone().into())) * ((alpha.into() + beta_challenges[0].into().double() + AB::ExprEF::from(main_current[1].clone().into()) * beta_challenges[1].into()) * (AB::ExprEF::ONE - AB::ExprEF::from(main_current[2].clone().into())) + AB::ExprEF::from(main_current[2].clone().into())) * AB::ExprEF::from(aux_current[0].clone().into()) - ((alpha.into() + beta_challenges[0].into() + (AB::ExprEF::from_u64(3) + AB::ExprEF::from(main_current[1].clone().into())) * beta_challenges[1].into() + AB::ExprEF::from(main_current[1].clone().into()) * beta_challenges[2].into()) * AB::ExprEF::from(main_current[3].clone().into()) + AB::ExprEF::ONE - AB::ExprEF::from(main_current[3].clone().into())) * ((alpha.into() + beta_challenges[0].into().double() + AB::ExprEF::from(main_current[0].clone().into()) * beta_challenges[1].into()) * (AB::ExprEF::ONE - AB::ExprEF::from(main_current[3].clone().into())) + AB::ExprEF::from(main_current[3].clone().into())) * AB::ExprEF::from(aux_next[0].clone().into()));
        builder.when_transition().assert_zero_ext((alpha.into() + AB::ExprEF::from_u64(3) * beta_challenges[0].into() + AB::ExprEF::from(main_current[0].clone().into()) * beta_challenges[1].into()) * (alpha.into() + AB::ExprEF::from_u64(3) * beta_challenges[0].into() + AB::ExprEF::from(main_current[0].clone().into()) * beta_challenges[1].into()) * (alpha.into() + AB::ExprEF::from_u64(3) * beta_challenges[0].into() + AB::ExprEF::from(main_current[1].clone().into()) * beta_challenges[1].into()) * AB::ExprEF::from(aux_current[1].clone().into()) + (alpha.into() + AB::ExprEF::from_u64(3) * beta_challenges[0].into() + AB::ExprEF::from(main_current[0].clone().into()) * beta_challenges[1].into()) * (alpha.into() + AB::ExprEF::from_u64(3) * beta_challenges[0].into() + AB::ExprEF::from(main_current[1].clone().into()) * beta_challenges[1].into()) * AB::ExprEF::from(main_current[4].clone().into()) + (alpha.into() + AB::ExprEF::from_u64(3) * beta_challenges[0].into() + AB::ExprEF::from(main_current[0].clone().into()) * beta_challenges[1].into()) * (alpha.into() + AB::ExprEF::from_u64(3) * beta_challenges[0].into() + AB::ExprEF::from(main_current[1].clone().into()) * beta_challenges[1].into()) * AB::ExprEF::from(main_current[5].clone().into()) - ((alpha.into() + AB::ExprEF::from_u64(3) * beta_challenges[0].into() + AB::ExprEF::from(main_current[0].clone().into()) * beta_challenges[1].into()) * (alpha.into() + AB::ExprEF::from_u64(3) * beta_challenges[0].into() + AB::ExprEF::from(main_current[0].clone().into()) * beta_challenges[1].into()) * (alpha.into() + AB::ExprEF::from_u64(3) * beta_challenges[0].into() + AB::ExprEF::from(main_current[1].clone().into()) * beta_challenges[1].into()) * AB::ExprEF::from(aux_next[1].clone().into()) + (alpha.into() + AB::ExprEF::from_u64(3) * beta_challenges[0].into() + AB::ExprEF::from(main_current[0].clone().into()) * beta_challenges[1].into()) * (alpha.into() + AB::ExprEF::from_u64(3) * beta_challenges[0].into() + AB::ExprEF::from(main_current[0].clone().into()) * beta_challenges[1].into()) * AB::ExprEF::from(main_current[6].clone().into())));
    }
}

impl BusesAir {
    fn buses_initial_values<F, EF, AB>() -> Vec<AB::ExprEF>
    where F: Field,
          EF: ExtensionField<F>,
          AB: MidenAirBuilder<F = F, EF = EF>,
    {
        vec![
            AB::ExprEF::ONE,
            AB::ExprEF::ZERO,
        ]
    }

    fn buses_transitions<F, EF, AB>(main: &RowMajorMatrix<F>, challenges: &[EF], periodic_evals: &[F], aux_current: RowMajorMatrixView<EF>) -> Vec<EF>
    where F: Field,
          EF: ExtensionField<F>,
          AB: MidenAirBuilder<F = F, EF = EF>,
    {
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        let aux_current = aux_current.row_slice(0).unwrap();
        let (&alpha, beta_challenges) = challenges.split_first().unwrap();
        let beta_challenges: [_; NUM_BETA_CHALLENGES] = beta_challenges.try_into().expect("Wrong number of randomness");
        let periodic_values: [_; NUM_PERIODIC_VALUES] = periodic_evals.try_into().expect("Wrong number of periodic values");
        vec![
            ((alpha + beta_challenges[0] + (AB::EF::from_u64(3) + AB::EF::from(main_current[1].clone())) * beta_challenges[1] + AB::EF::from(main_current[0].clone()) * beta_challenges[2]) * AB::EF::from(main_current[2].clone()) + AB::EF::ONE - AB::EF::from(main_current[2].clone())) * ((alpha + beta_challenges[0].double() + AB::EF::from(main_current[1].clone()) * beta_challenges[1]) * (AB::EF::ONE - AB::EF::from(main_current[2].clone())) + AB::EF::from(main_current[2].clone())) * AB::EF::from(aux_current[0].clone()) * (((alpha + beta_challenges[0] + (AB::EF::from_u64(3) + AB::EF::from(main_current[1].clone())) * beta_challenges[1] + AB::EF::from(main_current[1].clone()) * beta_challenges[2]) * AB::EF::from(main_current[3].clone()) + AB::EF::ONE - AB::EF::from(main_current[3].clone())) * ((alpha + beta_challenges[0].double() + AB::EF::from(main_current[0].clone()) * beta_challenges[1]) * (AB::EF::ONE - AB::EF::from(main_current[3].clone())) + AB::EF::from(main_current[3].clone()))).inverse(),
            (alpha + AB::EF::from_u64(3) * beta_challenges[0] + AB::EF::from(main_current[0].clone()) * beta_challenges[1]) * (alpha + AB::EF::from_u64(3) * beta_challenges[0] + AB::EF::from(main_current[0].clone()) * beta_challenges[1]) * (alpha + AB::EF::from_u64(3) * beta_challenges[0] + AB::EF::from(main_current[1].clone()) * beta_challenges[1]) * AB::EF::from(aux_current[1].clone()) + (alpha + AB::EF::from_u64(3) * beta_challenges[0] + AB::EF::from(main_current[0].clone()) * beta_challenges[1]) * (alpha + AB::EF::from_u64(3) * beta_challenges[0] + AB::EF::from(main_current[1].clone()) * beta_challenges[1]) * AB::EF::from(main_current[4].clone()) + (alpha + AB::EF::from_u64(3) * beta_challenges[0] + AB::EF::from(main_current[0].clone()) * beta_challenges[1]) * (alpha + AB::EF::from_u64(3) * beta_challenges[0] + AB::EF::from(main_current[1].clone()) * beta_challenges[1]) * AB::EF::from(main_current[5].clone()) - (alpha + AB::EF::from_u64(3) * beta_challenges[0] + AB::EF::from(main_current[0].clone()) * beta_challenges[1]) * (alpha + AB::EF::from_u64(3) * beta_challenges[0] + AB::EF::from(main_current[0].clone()) * beta_challenges[1]) * AB::EF::from(main_current[6].clone()) * ((alpha + AB::EF::from_u64(3) * beta_challenges[0] + AB::EF::from(main_current[0].clone()) * beta_challenges[1]) * (alpha + AB::EF::from_u64(3) * beta_challenges[0] + AB::EF::from(main_current[0].clone()) * beta_challenges[1]) * (alpha + AB::EF::from_u64(3) * beta_challenges[0] + AB::EF::from(main_current[1].clone()) * beta_challenges[1])).inverse(),
        ]
    }
}