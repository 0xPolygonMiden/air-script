use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir, BaseAirWithPublicValues};
use p3_matrix::Matrix;
use p3_field::PrimeCharacteristicRing;
use crate::helpers::{AirBuilderWithPeriodicColumns, BaseAirWithPeriodicColumns};

pub const NUM_COLUMNS: usize = 8;

pub const NUM_PUBLIC_VALUES: usize = 16;

pub struct ComputedIndicesAir;

impl<F> BaseAir<F> for ComputedIndicesAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for ComputedIndicesAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: PrimeCharacteristicRing> BaseAirWithPeriodicColumns<F> for ComputedIndicesAir {
    fn get_periodic_columns(&self) -> Vec<Vec<F>> {
        vec![
        ]
    }
}

impl<AB: AirBuilderWithPublicValues + AirBuilderWithPeriodicColumns> Air<AB> for ComputedIndicesAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values = builder.periodic_columns();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        builder.when_first_row().assert_zero::<_>(main_current[0].clone().into());
        builder.assert_zero::<_>(main_current[0].clone().into());
        builder.assert_zero::<_>(main_current[1].clone().into() - AB::Expr::from_u64(2));
        builder.assert_zero::<_>(main_current[2].clone().into() - AB::Expr::from_u64(4));
        builder.assert_zero::<_>(main_current[3].clone().into() - AB::Expr::from_u64(6));
        builder.when_transition().assert_zero::<_>(main_next[4].clone().into());
        builder.when_transition().assert_zero::<_>(main_next[5].clone().into() - main_current[5].clone().into().double());
        builder.when_transition().assert_zero::<_>(main_next[6].clone().into() - AB::Expr::from_u64(6) * main_current[6].clone().into());
        builder.when_transition().assert_zero::<_>(main_next[7].clone().into() - AB::Expr::from_u64(12) * main_current[7].clone().into());
    }
}