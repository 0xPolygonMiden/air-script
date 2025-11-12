use p3_air::{Air, BaseAir, BaseAirWithPublicValues, AirBuilder, ExtensionBuilder};
use p3_matrix::Matrix;
use p3_field::{Field, PrimeCharacteristicRing};
use crate::test_utils::plonky3_traits::{AirScriptAir, AirScriptBuilder};

pub const MAIN_WIDTH: usize = 1;
pub const AUX_WIDTH: usize = 2;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 6;
pub const NUM_ALPHA_CHALLENGES: usize = 2;

pub struct BusesAir;

impl<F> BaseAir<F> for BusesAir {
    fn width(&self) -> usize {
        MAIN_WIDTH
    }
}

impl<F> BaseAirWithPublicValues<F> for BusesAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: Field, AB: AirScriptBuilder<F = F>> AirScriptAir<F, AB> for BusesAir {
    fn aux_width(&self) -> usize {
        AUX_WIDTH
    }

    fn num_alpha_challenges(&self) -> usize {
        NUM_ALPHA_CHALLENGES
    }

    fn periodic_table(&self) -> Vec<Vec<F>> {
        vec![]
    }

    fn eval(&self, builder: &mut AB) {
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values: [_; NUM_PERIODIC_VALUES] = builder.periodic_evals().try_into().expect("Wrong number of periodic values");
        let main = builder.main();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        let alpha_challenges: [_; NUM_ALPHA_CHALLENGES] = builder.alpha_powers().try_into().expect("Wrong number of alpha challenges");
        let beta = builder.beta();
        let aux_bus_boundary_values: [_; AUX_WIDTH] = builder.aux_bus_boundary_values().try_into().expect("Wrong number of aux bus boundary values");
        let aux = builder.permutation();
        let (aux_current, aux_next) = (
            aux.row_slice(0).unwrap(),
            aux.row_slice(1).unwrap(),
        );

        // Main boundary constraints

        // Main integrity/transition constraints

        // Aux boundary constraints
        builder.when_first_row().assert_zero_ext(AB::ExprEF::from(aux_current[0].clone().into()) - aux_bus_boundary_values[0].into());
        builder.when_first_row().assert_zero_ext(AB::ExprEF::from(aux_current[1].clone().into()));
        builder.when_last_row().assert_zero_ext(AB::ExprEF::from(aux_current[1].clone().into()) - aux_bus_boundary_values[1].into());

        // Aux integrity/transition constraints
        builder.when_transition().assert_zero_ext(((beta.into() + alpha_challenges[0].into()) * AB::ExprEF::from(main_current[0].clone().into()) + AB::ExprEF::ONE - AB::ExprEF::from(main_current[0].clone().into())) * AB::ExprEF::from(aux_current[0].clone().into()) - ((beta.into() + alpha_challenges[0].into()) * (AB::ExprEF::from(main_current[0].clone().into()) - AB::ExprEF::ONE) + AB::ExprEF::ONE - (AB::ExprEF::from(main_current[0].clone().into()) - AB::ExprEF::ONE)) * AB::ExprEF::from(aux_next[0].clone().into()));
        builder.when_transition().assert_zero_ext((beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * (beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * (beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * AB::ExprEF::from(aux_current[1].clone().into()) + (beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * (beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * AB::ExprEF::from(main_current[0].clone().into()) + (beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * (beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * AB::ExprEF::from(main_current[0].clone().into()) - ((beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * (beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * (beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * AB::ExprEF::from(aux_next[1].clone().into()) + (beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * (beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()).double()));
    }
}

impl<AB: AirScriptBuilder> Air<AB> for BusesAir {
    fn eval(&self, builder: &mut AB) {
        <Self as AirScriptAir<AB::F, AB>>::eval(self, builder);
    }
}