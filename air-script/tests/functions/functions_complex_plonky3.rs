use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir, BaseAirWithPublicValues};
use p3_matrix::Matrix;
use p3_field::PrimeCharacteristicRing;
use crate::helpers::{AirBuilderWithPeriodicColumns, BaseAirWithPeriodicColumns};

pub const NUM_COLUMNS: usize = 17;

pub const NUM_PUBLIC_VALUES: usize = 16;

pub struct FunctionsAir;

impl<F> BaseAir<F> for FunctionsAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for FunctionsAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: PrimeCharacteristicRing> BaseAirWithPeriodicColumns<F> for FunctionsAir {
    fn get_periodic_columns(&self) -> Vec<Vec<F>> {
        vec![
        ]
    }
}

impl<AB: AirBuilderWithPublicValues + AirBuilderWithPeriodicColumns> Air<AB> for FunctionsAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values = builder.periodic_columns();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        builder.when_first_row().assert_zero::<_>(main_current[3]);
        builder.when_transition().assert_zero::<_>(main_next[16] - main_current[16] * ((main_current[3] * main_current[3] * main_current[3] * main_current[3] * main_current[3] * main_current[3] * main_current[3] * main_current[1] * main_current[2] + main_current[3] * main_current[3] * (AB::Expr::from(AB::F::from_u64(1)) - main_current[1]) * main_current[2] + main_current[3] * main_current[1] * (AB::Expr::from(AB::F::from_u64(1)) - main_current[2]) + (AB::Expr::from(AB::F::from_u64(1)) - main_current[1]) * (AB::Expr::from(AB::F::from_u64(1)) - main_current[2])) * main_current[0] - main_current[0] + AB::Expr::from(AB::F::from_u64(1))));
        builder.when_transition().assert_zero::<_>(main_next[3] - (main_current[4] + main_current[5] + main_current[6] + main_current[7] + main_current[8] + main_current[9] + main_current[10] + main_current[11] + main_current[12] + main_current[13] + main_current[14] + main_current[15] + AB::Expr::from(AB::F::from_u64(1))) * AB::Expr::from(AB::F::from_u64(2)));
    }
}