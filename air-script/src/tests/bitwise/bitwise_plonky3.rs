use p3_air::{Air, BaseAir, BaseAirWithPublicValues, AirBuilder, ExtensionBuilder};
use p3_matrix::Matrix;
use p3_field::{Field, PrimeCharacteristicRing};
use crate::test_utils::plonky3_traits::{AirScriptAir, AirScriptBuilder};

pub const MAIN_WIDTH: usize = 14;
pub const AUX_WIDTH: usize = 0;
pub const NUM_PERIODIC_VALUES: usize = 2;
pub const PERIOD: usize = 8;
pub const NUM_PUBLIC_VALUES: usize = 16;
pub const NUM_BETA_CHALLENGES: usize = 0;

pub struct BitwiseAir;

impl<F> BaseAir<F> for BitwiseAir {
    fn width(&self) -> usize {
        MAIN_WIDTH
    }
}

impl<F> BaseAirWithPublicValues<F> for BitwiseAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: Field, AB: AirScriptBuilder<F = F>> AirScriptAir<F, AB> for BitwiseAir {
    fn aux_width(&self) -> usize {
        AUX_WIDTH
    }

    fn num_beta_challenges(&self) -> usize {
        NUM_BETA_CHALLENGES
    }

    fn periodic_table(&self) -> Vec<Vec<F>> {
        vec![
            vec![F::from_u64(1), F::from_u64(0), F::from_u64(0), F::from_u64(0), F::from_u64(0), F::from_u64(0), F::from_u64(0), F::from_u64(0)],
            vec![F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(0)],
        ]
    }

    fn eval(&self, builder: &mut AB) {
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values: [_; NUM_PERIODIC_VALUES] = builder.periodic_evals().try_into().expect("Wrong number of periodic values");
        let main = builder.main();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );

        // Main boundary constraints
        builder.when_first_row().assert_zero(main_current[13].clone().into());

        // Main integrity/transition constraints
        builder.assert_zero(main_current[0].clone().into() * main_current[0].clone().into() - main_current[0].clone().into());
        builder.when_transition().assert_zero_ext(periodic_values[1].into() * (AB::ExprEF::from(main_next[0].clone().into()) - AB::ExprEF::from(main_current[0].clone().into())));
        builder.assert_zero(main_current[3].clone().into() * main_current[3].clone().into() - main_current[3].clone().into());
        builder.assert_zero(main_current[4].clone().into() * main_current[4].clone().into() - main_current[4].clone().into());
        builder.assert_zero(main_current[5].clone().into() * main_current[5].clone().into() - main_current[5].clone().into());
        builder.assert_zero(main_current[6].clone().into() * main_current[6].clone().into() - main_current[6].clone().into());
        builder.assert_zero(main_current[7].clone().into() * main_current[7].clone().into() - main_current[7].clone().into());
        builder.assert_zero(main_current[8].clone().into() * main_current[8].clone().into() - main_current[8].clone().into());
        builder.assert_zero(main_current[9].clone().into() * main_current[9].clone().into() - main_current[9].clone().into());
        builder.assert_zero(main_current[10].clone().into() * main_current[10].clone().into() - main_current[10].clone().into());
        builder.assert_zero_ext(periodic_values[0].into() * (AB::ExprEF::from(main_current[1].clone().into()) - (AB::ExprEF::from(main_current[3].clone().into()) + AB::ExprEF::from(main_current[4].clone().into()).double() + AB::ExprEF::from_u64(4) * AB::ExprEF::from(main_current[5].clone().into()) + AB::ExprEF::from_u64(8) * AB::ExprEF::from(main_current[6].clone().into()))));
        builder.assert_zero_ext(periodic_values[0].into() * (AB::ExprEF::from(main_current[2].clone().into()) - (AB::ExprEF::from(main_current[7].clone().into()) + AB::ExprEF::from(main_current[8].clone().into()).double() + AB::ExprEF::from_u64(4) * AB::ExprEF::from(main_current[9].clone().into()) + AB::ExprEF::from_u64(8) * AB::ExprEF::from(main_current[10].clone().into()))));
        builder.when_transition().assert_zero_ext(periodic_values[1].into() * (AB::ExprEF::from(main_next[1].clone().into()) - (AB::ExprEF::from(main_current[1].clone().into()) * AB::ExprEF::from_u64(16) + AB::ExprEF::from(main_current[3].clone().into()) + AB::ExprEF::from(main_current[4].clone().into()).double() + AB::ExprEF::from_u64(4) * AB::ExprEF::from(main_current[5].clone().into()) + AB::ExprEF::from_u64(8) * AB::ExprEF::from(main_current[6].clone().into()))));
        builder.when_transition().assert_zero_ext(periodic_values[1].into() * (AB::ExprEF::from(main_next[2].clone().into()) - (AB::ExprEF::from(main_current[2].clone().into()) * AB::ExprEF::from_u64(16) + AB::ExprEF::from(main_current[7].clone().into()) + AB::ExprEF::from(main_current[8].clone().into()).double() + AB::ExprEF::from_u64(4) * AB::ExprEF::from(main_current[9].clone().into()) + AB::ExprEF::from_u64(8) * AB::ExprEF::from(main_current[10].clone().into()))));
        builder.assert_zero_ext(periodic_values[0].into() * AB::ExprEF::from(main_current[11].clone().into()));
        builder.when_transition().assert_zero_ext(periodic_values[1].into() * (AB::ExprEF::from(main_current[12].clone().into()) - AB::ExprEF::from(main_next[11].clone().into())));
        builder.assert_zero((AB::Expr::ONE - main_current[0].clone().into()) * (main_current[12].clone().into() - (main_current[11].clone().into() * AB::Expr::from_u64(16) + main_current[3].clone().into() * main_current[7].clone().into() + main_current[4].clone().into().double() * main_current[8].clone().into() + AB::Expr::from_u64(4) * main_current[5].clone().into() * main_current[9].clone().into() + AB::Expr::from_u64(8) * main_current[6].clone().into() * main_current[10].clone().into())) + main_current[0].clone().into() * (main_current[12].clone().into() - (main_current[11].clone().into() * AB::Expr::from_u64(16) + main_current[3].clone().into() + main_current[7].clone().into() - main_current[3].clone().into().double() * main_current[7].clone().into() + (main_current[4].clone().into() + main_current[8].clone().into() - main_current[4].clone().into().double() * main_current[8].clone().into()).double() + AB::Expr::from_u64(4) * (main_current[5].clone().into() + main_current[9].clone().into() - main_current[5].clone().into().double() * main_current[9].clone().into()) + AB::Expr::from_u64(8) * (main_current[6].clone().into() + main_current[10].clone().into() - main_current[6].clone().into().double() * main_current[10].clone().into()))));

        // Aux boundary constraints

        // Aux integrity/transition constraints
    }
}

impl<AB: AirScriptBuilder> Air<AB> for BitwiseAir {
    fn eval(&self, builder: &mut AB) {
        <Self as AirScriptAir<AB::F, AB>>::eval(self, builder);
    }
}