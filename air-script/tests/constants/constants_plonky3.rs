use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir, BaseAirWithPublicValues};
use p3_matrix::Matrix;
use p3_field::PrimeCharacteristicRing;
use crate::helpers::{AirBuilderWithPeriodicColumns, BaseAirWithPeriodicColumns};

pub const NUM_COLUMNS: usize = 7;

pub const NUM_PUBLIC_VALUES: usize = 32;

pub struct ConstantsAir;

impl<F> BaseAir<F> for ConstantsAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for ConstantsAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: PrimeCharacteristicRing> BaseAirWithPeriodicColumns<F> for ConstantsAir {
    fn get_periodic_columns(&self) -> Vec<Vec<F>> {
        vec![
        ]
    }
}

impl<AB: AirBuilderWithPublicValues + AirBuilderWithPeriodicColumns> Air<AB> for ConstantsAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values = builder.periodic_columns();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        builder.when_first_row().assert_zero::<_>(main_current[0].clone().into() - AB::Expr::ONE);
        builder.when_first_row().assert_zero::<_>(main_current[1].clone().into() - AB::Expr::ONE);
        builder.when_first_row().assert_zero::<_>(main_current[2].clone().into());
        builder.when_first_row().assert_zero::<_>(main_current[3].clone().into() - AB::Expr::ONE);
        builder.when_first_row().assert_zero::<_>(main_current[4].clone().into() - AB::Expr::ONE);
        builder.when_last_row().assert_zero::<_>(main_current[6].clone().into());
        builder.when_transition().assert_zero::<_>(main_next[0].clone().into() - (main_current[0].clone().into() + AB::Expr::ONE));
        builder.when_transition().assert_zero::<_>(main_next[1].clone().into());
        builder.when_transition().assert_zero::<_>(main_next[2].clone().into() - main_current[2].clone().into());
        builder.when_transition().assert_zero::<_>(main_next[5].clone().into() - (main_current[5].clone().into() + AB::Expr::ONE));
        builder.assert_zero::<_>(main_current[4].clone().into() - AB::Expr::ONE);
    }
}