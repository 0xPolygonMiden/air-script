use p3_air::{Air, BaseAir, BaseAirWithPublicValues, AirBuilder, ExtensionBuilder};
use p3_matrix::Matrix;
use p3_field::{Field, PrimeCharacteristicRing};
use crate::test_utils::plonky3_traits::{AirScriptAir, AirScriptBuilder};

pub const MAIN_WIDTH: usize = 16;
pub const AUX_WIDTH: usize = 0;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 16;
pub const NUM_BETA_CHALLENGES: usize = 0;

pub struct ListComprehensionAir;

impl<F> BaseAir<F> for ListComprehensionAir {
    fn width(&self) -> usize {
        MAIN_WIDTH
    }
}

impl<F> BaseAirWithPublicValues<F> for ListComprehensionAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: Field, AB: AirScriptBuilder<F = F>> AirScriptAir<F, AB> for ListComprehensionAir {
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
        builder.when_first_row().assert_zero(main_current[10].clone().into());

        // Main integrity/transition constraints
        builder.assert_zero(main_current[0].clone().into() - main_current[2].clone().into());
        builder.assert_zero(main_current[4].clone().into() - main_current[0].clone().into() * AB::Expr::from_u64(8) * main_current[11].clone().into());
        builder.when_transition().assert_zero(main_current[4].clone().into() - main_current[0].clone().into() * (main_next[8].clone().into() - main_next[12].clone().into()));
        builder.assert_zero(main_current[6].clone().into() - main_current[0].clone().into() * (main_current[9].clone().into() - main_current[14].clone().into()));
        builder.assert_zero(main_current[1].clone().into() - (main_current[5].clone().into() - main_current[8].clone().into() - main_current[12].clone().into() + AB::Expr::from_u64(10) + main_current[6].clone().into() - main_current[9].clone().into() - main_current[13].clone().into() + AB::Expr::from_u64(20) + main_current[7].clone().into() - main_current[10].clone().into() - main_current[14].clone().into()));
        builder.assert_zero(main_current[14].clone().into() - AB::Expr::from_u64(10));

        // Aux boundary constraints

        // Aux integrity/transition constraints
    }
}

impl<AB: AirScriptBuilder> Air<AB> for ListComprehensionAir {
    fn eval(&self, builder: &mut AB) {
        <Self as AirScriptAir<AB::F, AB>>::eval(self, builder);
    }
}