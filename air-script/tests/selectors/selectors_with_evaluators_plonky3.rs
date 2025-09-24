use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir, BaseAirWithPublicValues};
use p3_matrix::Matrix;
use p3_field::PrimeCharacteristicRing;
use crate::helpers::{AirBuilderWithPeriodicColumns, BaseAirWithPeriodicColumns};

pub const NUM_COLUMNS: usize = 4;

pub const NUM_PUBLIC_VALUES: usize = 16;

pub struct SelectorsAir;

impl<F> BaseAir<F> for SelectorsAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for SelectorsAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: PrimeCharacteristicRing> BaseAirWithPeriodicColumns<F> for SelectorsAir {
    fn get_periodic_columns(&self) -> Vec<Vec<F>> {
        vec![
        ]
    }
}

impl<AB: AirBuilderWithPublicValues + AirBuilderWithPeriodicColumns> Air<AB> for SelectorsAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values = builder.periodic_columns();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        builder.when_first_row().assert_zero::<_>(main_current[3]);
        builder.when_transition().assert_zero::<_>(main_current[0] * (AB::Expr::from(AB::F::from_u64(1)) - main_current[1]) * main_next[3]);
        builder.when_transition().assert_zero::<_>(main_current[1] * main_current[2] * main_current[0] * (main_next[3] - main_current[3]) + (AB::Expr::from(AB::F::from_u64(1)) - main_current[1]) * (AB::Expr::from(AB::F::from_u64(1)) - main_current[2]) * (main_next[3] - AB::Expr::from(AB::F::from_u64(1))));
    }
}