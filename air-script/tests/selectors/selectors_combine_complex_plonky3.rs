use p3_air::{Air, BaseAir, BaseAirWithPublicValues, AirBuilder, ExtensionBuilder};
use p3_matrix::Matrix;
use p3_field::{Field, PrimeCharacteristicRing};
use crate::helpers::{AirScriptAir, AirScriptBuilder};

pub const MAIN_WIDTH: usize = 6;
pub const AUX_WIDTH: usize = 1;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 1;
pub const NUM_ALPHA_CHALLENGES: usize = 2;

pub struct SelectorsAir;

impl<F> BaseAir<F> for SelectorsAir {
    fn width(&self) -> usize {
        MAIN_WIDTH
    }
}

impl<F> BaseAirWithPublicValues<F> for SelectorsAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: Field, AB: AirScriptBuilder<F = F>> AirScriptAir<F, AB> for SelectorsAir {
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
        builder.when_first_row().assert_zero(main_current[5].clone().into());

        // Main integrity/transition constraints
        builder.assert_zero((main_current[0].clone().into() + (AB::Expr::ONE - main_current[0].clone().into()) * main_current[1].clone().into()) * (main_current[3].clone().into() - AB::Expr::from_u64(16)) + (AB::Expr::ONE - main_current[0].clone().into()) * (AB::Expr::ONE - main_current[1].clone().into()) * (main_current[4].clone().into() - AB::Expr::from_u64(5)));
        builder.assert_zero((AB::Expr::ONE - main_current[0].clone().into()) * (main_current[5].clone().into() - AB::Expr::from_u64(5)) + main_current[0].clone().into() * (main_current[4].clone().into() - AB::Expr::from_u64(4)));
        builder.assert_zero(main_current[0].clone().into() * (main_current[5].clone().into() - AB::Expr::from_u64(20)) + (AB::Expr::ONE - main_current[0].clone().into()) * main_current[1].clone().into() * (main_current[4].clone().into() - AB::Expr::from_u64(31)));

        // Aux boundary constraints
        builder.when_first_row().assert_zero_ext(AB::ExprEF::from(aux_current[0].clone().into()) - AB::ExprEF::ONE);
        builder.when_last_row().assert_zero_ext(AB::ExprEF::from(aux_current[0].clone().into()) - AB::ExprEF::ONE);

        // Aux integrity/transition constraints
        builder.when_transition().assert_zero_ext(((beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * AB::ExprEF::from(main_current[0].clone().into()) * AB::ExprEF::from(main_current[5].clone().into()) + AB::ExprEF::ONE - AB::ExprEF::from(main_current[0].clone().into()) * AB::ExprEF::from(main_current[5].clone().into())) * ((beta.into() + alpha_challenges[0].into() + alpha_challenges[1].into().double()) * (AB::ExprEF::ONE - AB::ExprEF::from(main_current[0].clone().into())) * AB::ExprEF::from(main_current[1].clone().into()) * AB::ExprEF::from(main_current[5].clone().into()) + AB::ExprEF::ONE - (AB::ExprEF::ONE - AB::ExprEF::from(main_current[0].clone().into())) * AB::ExprEF::from(main_current[1].clone().into()) * AB::ExprEF::from(main_current[5].clone().into())) * AB::ExprEF::from(aux_current[0].clone().into()) - ((beta.into() + AB::ExprEF::from_u64(3) * alpha_challenges[0].into() + AB::ExprEF::from_u64(4) * alpha_challenges[1].into()) * (AB::ExprEF::ONE - AB::ExprEF::from(main_current[0].clone().into())) * (AB::ExprEF::ONE - AB::ExprEF::from(main_current[1].clone().into())) * AB::ExprEF::from(main_current[4].clone().into()) + AB::ExprEF::ONE - (AB::ExprEF::ONE - AB::ExprEF::from(main_current[0].clone().into())) * (AB::ExprEF::ONE - AB::ExprEF::from(main_current[1].clone().into())) * AB::ExprEF::from(main_current[4].clone().into())) * AB::ExprEF::from(aux_next[0].clone().into()));
    }
}

impl<AB: AirScriptBuilder> Air<AB> for SelectorsAir {
    fn eval(&self, builder: &mut AB) {
        <Self as AirScriptAir<AB::F, AB>>::eval(self, builder);
    }
}