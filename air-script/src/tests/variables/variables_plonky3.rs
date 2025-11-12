use p3_air::{Air, BaseAir, BaseAirWithPublicValues, AirBuilder, ExtensionBuilder};
use p3_matrix::Matrix;
use p3_field::{Field, PrimeCharacteristicRing};
use crate::test_utils::plonky3_traits::{AirScriptAir, AirScriptBuilder};

pub const MAIN_WIDTH: usize = 4;
pub const AUX_WIDTH: usize = 0;
pub const NUM_PERIODIC_VALUES: usize = 1;
pub const PERIOD: usize = 8;
pub const NUM_PUBLIC_VALUES: usize = 32;
pub const NUM_ALPHA_CHALLENGES: usize = 0;

pub struct VariablesAir;

impl<F> BaseAir<F> for VariablesAir {
    fn width(&self) -> usize {
        MAIN_WIDTH
    }
}

impl<F> BaseAirWithPublicValues<F> for VariablesAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: Field, AB: AirScriptBuilder<F = F>> AirScriptAir<F, AB> for VariablesAir {
    fn aux_width(&self) -> usize {
        AUX_WIDTH
    }

    fn num_alpha_challenges(&self) -> usize {
        NUM_ALPHA_CHALLENGES
    }

    fn periodic_table(&self) -> Vec<Vec<F>> {
        vec![
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
        builder.when_first_row().assert_zero(main_current[1].clone().into());
        builder.when_last_row().assert_zero(main_current[1].clone().into() - AB::Expr::ONE);

        // Main integrity/transition constraints
        builder.assert_zero(main_current[0].clone().into() * main_current[0].clone().into() - main_current[0].clone().into());
        builder.when_transition().assert_zero_ext(periodic_values[0].into() * (AB::ExprEF::from(main_next[0].clone().into()) - AB::ExprEF::from(main_current[0].clone().into())));
        builder.assert_zero((AB::Expr::ONE - main_current[0].clone().into()) * (main_current[3].clone().into() - main_current[1].clone().into() - main_current[2].clone().into()) - (AB::Expr::from_u64(6) - (AB::Expr::from_u64(7) - main_current[0].clone().into())));
        builder.when_transition().assert_zero(main_current[0].clone().into() * (main_current[3].clone().into() - main_current[1].clone().into() * main_current[2].clone().into()) - (AB::Expr::ONE - main_next[0].clone().into()));

        // Aux boundary constraints

        // Aux integrity/transition constraints
    }
}

impl<AB: AirScriptBuilder> Air<AB> for VariablesAir {
    fn eval(&self, builder: &mut AB) {
        <Self as AirScriptAir<AB::F, AB>>::eval(self, builder);
    }
}