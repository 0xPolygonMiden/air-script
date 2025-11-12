use p3_air::{Air, BaseAir, BaseAirWithPublicValues, AirBuilder, ExtensionBuilder};
use p3_matrix::Matrix;
use p3_field::{Field, PrimeCharacteristicRing};
use crate::test_utils::plonky3_traits::{AirScriptAir, AirScriptBuilder};

pub const MAIN_WIDTH: usize = 7;
pub const AUX_WIDTH: usize = 0;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 32;
pub const NUM_BETA_CHALLENGES: usize = 0;

pub struct ConstantsAir;

impl<F> BaseAir<F> for ConstantsAir {
    fn width(&self) -> usize {
        MAIN_WIDTH
    }
}

impl<F> BaseAirWithPublicValues<F> for ConstantsAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: Field, AB: AirScriptBuilder<F = F>> AirScriptAir<F, AB> for ConstantsAir {
    fn aux_width(&self) -> usize {
        AUX_WIDTH
    }

    fn num_beta_challenges(&self) -> usize {
        NUM_BETA_CHALLENGES
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

        // Main boundary constraints
        builder.when_first_row().assert_zero(main_current[0].clone().into() - AB::Expr::ONE);
        builder.when_first_row().assert_zero(main_current[1].clone().into() - AB::Expr::ONE);
        builder.when_first_row().assert_zero(main_current[2].clone().into());
        builder.when_first_row().assert_zero(main_current[3].clone().into() - AB::Expr::ONE);
        builder.when_first_row().assert_zero(main_current[4].clone().into() - AB::Expr::ONE);
        builder.when_last_row().assert_zero(main_current[6].clone().into());

        // Main integrity/transition constraints
        builder.when_transition().assert_zero(main_next[0].clone().into() - (main_current[0].clone().into() + AB::Expr::ONE));
        builder.when_transition().assert_zero(main_next[1].clone().into());
        builder.when_transition().assert_zero(main_next[2].clone().into() - main_current[2].clone().into());
        builder.when_transition().assert_zero(main_next[5].clone().into() - (main_current[5].clone().into() + AB::Expr::ONE));
        builder.assert_zero(main_current[4].clone().into() - AB::Expr::ONE);

        // Aux boundary constraints

        // Aux integrity/transition constraints
    }
}

impl<AB: AirScriptBuilder> Air<AB> for ConstantsAir {
    fn eval(&self, builder: &mut AB) {
        <Self as AirScriptAir<AB::F, AB>>::eval(self, builder);
    }
}