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
        builder.when_first_row().assert_zero::<_>(main_current[3].into());
        builder.when_transition().assert_zero::<_>(main_next[16].into() - main_current[16].into() * ((main_current[3].into() * main_current[3].into() * main_current[3].into() * main_current[3].into() * main_current[3].into() * main_current[3].into() * main_current[3].into() * main_current[1].into() * main_current[2].into() + main_current[3].into() * main_current[3].into() * (AB::Expr::ONE - main_current[1].into()) * main_current[2].into() + main_current[3].into() * main_current[1].into() * (AB::Expr::ONE - main_current[2].into()) + (AB::Expr::ONE - main_current[1].into()) * (AB::Expr::ONE - main_current[2].into())) * main_current[0].into() - main_current[0].into() + AB::Expr::ONE));
        builder.when_transition().assert_zero::<_>(main_next[3].into() - (main_current[4].into() + main_current[5].into() + main_current[6].into() + main_current[7].into() + main_current[8].into() + main_current[9].into() + main_current[10].into() + main_current[11].into() + main_current[12].into() + main_current[13].into() + main_current[14].into() + main_current[15].into() + AB::Expr::ONE).double());
    }
}