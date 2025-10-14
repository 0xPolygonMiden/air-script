use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir, BaseAirWithPublicValues};
use p3_matrix::Matrix;
use p3_field::PrimeCharacteristicRing;
use crate::helpers::{AirBuilderWithPeriodicColumns, BaseAirWithPeriodicColumns};

pub const NUM_COLUMNS: usize = 2;

pub const NUM_PUBLIC_VALUES: usize = 1;

pub struct ListComprehensionAir;

impl<F> BaseAir<F> for ListComprehensionAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for ListComprehensionAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: PrimeCharacteristicRing> BaseAirWithPeriodicColumns<F> for ListComprehensionAir {
    fn get_periodic_columns(&self) -> Vec<Vec<F>> {
        vec![
        ]
    }
}

impl<AB: AirBuilderWithPublicValues + AirBuilderWithPeriodicColumns> Air<AB> for ListComprehensionAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values = builder.periodic_columns();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        builder.when_first_row().assert_zero::<_>(main_current[0].into());
        builder.assert_zero::<_>(main_current[0].into() + main_current[1].into().double() - AB::Expr::from_u64(3));
        builder.assert_zero::<_>(main_current[0].into().double() + main_current[1].into() * AB::Expr::from_u64(3) - AB::Expr::from_u64(5));
        builder.assert_zero::<_>(main_current[0].into() * AB::Expr::from_u64(3) + main_current[1].into() * AB::Expr::from_u64(4) - AB::Expr::from_u64(7));
    }
}