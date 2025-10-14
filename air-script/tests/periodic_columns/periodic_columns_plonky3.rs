use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir, BaseAirWithPublicValues};
use p3_matrix::Matrix;
use p3_field::PrimeCharacteristicRing;
use crate::helpers::{AirBuilderWithPeriodicColumns, BaseAirWithPeriodicColumns};

pub const NUM_COLUMNS: usize = 3;

pub const NUM_PUBLIC_VALUES: usize = 16;

pub struct PeriodicColumnsAir;

impl<F> BaseAir<F> for PeriodicColumnsAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for PeriodicColumnsAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: PrimeCharacteristicRing> BaseAirWithPeriodicColumns<F> for PeriodicColumnsAir {
    fn get_periodic_columns(&self) -> Vec<Vec<F>> {
        vec![
            vec![F::from_u64(1), F::from_u64(0), F::from_u64(0), F::from_u64(0)],
            vec![F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(0)],
        ]
    }
}

impl<AB: AirBuilderWithPublicValues + AirBuilderWithPeriodicColumns> Air<AB> for PeriodicColumnsAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values = builder.periodic_columns();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        builder.when_first_row().assert_zero::<_>(main_current[0].into());
        builder.assert_zero::<_>(periodic_values[0].into() * (main_current[1].into() + main_current[2].into()));
        builder.when_transition().assert_zero::<_>(periodic_values[1].into() * (main_next[0].into() - main_current[0].into()));
    }
}