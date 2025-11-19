use p3_field::{ExtensionField, Field, PrimeCharacteristicRing};
use p3_matrix::Matrix;
use p3_miden_air::{MidenAir, MidenAirBuilder};
use crate::test_utils::plonky3_traits::AirScriptAir;

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

    fn eval<AB>(&self, builder: &mut AB)
    where AB: MidenAirBuilder<F = F, EF = EF>,
    {
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let preprocessed = builder.preprocessed();
        let periodic_values = preprocessed.row_slice(0).unwrap();
        let main = builder.main();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        let (alpha, beta_challenges) = builder.permutation_randomness().split_first().unwrap();
        let beta_challenges: [_; NUM_BETA_CHALLENGES] = beta_challenges.try_into().expect("Wrong number of beta challenges");
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

impl<F: Field, EF: ExtensionField<F>> AirScriptAir<F, EF> for BusesAir {
    fn num_beta_challenges(&self) -> usize {
        NUM_BETA_CHALLENGES
    }

    fn periodic_table(&self) -> Vec<Vec<F>> {
        vec![]
    }
}