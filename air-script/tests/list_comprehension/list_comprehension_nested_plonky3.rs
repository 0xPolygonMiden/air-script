use p3_air::{Air, BaseAir, BaseAirWithPublicValues, ExtensionBuilder};
use p3_matrix::Matrix;
use p3_field::{Field, PrimeCharacteristicRing};
use crate::helpers::{AirScriptAir, AirScriptBuilder};

pub const MAIN_WIDTH: usize = 2;
pub const AUX_WIDTH: usize = 0;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 1;
pub const NUM_ALPHA_CHALLENGES: usize = 0;

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

    fn num_alpha_challenges(&self) -> usize {
        NUM_ALPHA_CHALLENGES
    }

    fn periodic_table(&self) -> Vec<Vec<F>> {
        vec![]
    }

    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values: [_; NUM_PERIODIC_VALUES] = builder.periodic_evals().try_into().expect("Wrong number of periodic values");
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        builder.when_first_row().assert_zero_ext::<_>(main_current[0].clone().into());
        builder.assert_zero_ext::<_>(main_current[0].clone().into() + main_current[1].clone().into().double() - AB::Expr::from_u64(3));
        builder.assert_zero_ext::<_>(main_current[0].clone().into().double() + main_current[1].clone().into() * AB::Expr::from_u64(3) - AB::Expr::from_u64(5));
        builder.assert_zero_ext::<_>(main_current[0].clone().into() * AB::Expr::from_u64(3) + main_current[1].clone().into() * AB::Expr::from_u64(4) - AB::Expr::from_u64(7));
    }
}

impl<AB: AirScriptBuilder> Air<AB> for ListComprehensionAir {
    fn eval(&self, builder: &mut AB) {
        <Self as AirScriptAir<AB::F, AB>>::eval(self, builder);
    }
}