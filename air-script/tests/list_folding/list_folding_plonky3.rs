use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir, BaseAirWithPublicValues};
use p3_matrix::Matrix;
use p3_field::PrimeCharacteristicRing;
use crate::helpers::{AirBuilderWithPeriodicColumns, BaseAirWithPeriodicColumns};

pub const NUM_COLUMNS: usize = 17;

pub const NUM_PUBLIC_VALUES: usize = 16;

pub struct ListFoldingAir;

impl<F> BaseAir<F> for ListFoldingAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for ListFoldingAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: PrimeCharacteristicRing> BaseAirWithPeriodicColumns<F> for ListFoldingAir {
    fn get_periodic_columns(&self) -> Vec<Vec<F>> {
        vec![
        ]
    }
}

impl<AB: AirBuilderWithPublicValues + AirBuilderWithPeriodicColumns> Air<AB> for ListFoldingAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values = builder.periodic_columns();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        builder.when_first_row().assert_zero::<_>(main_current[11].clone().into());
        builder.when_transition().assert_zero::<_>(main_next[5].clone().into() - (main_current[9].clone().into() + main_current[10].clone().into() + main_current[11].clone().into() + main_current[12].clone().into() + main_current[13].clone().into() * main_current[14].clone().into() * main_current[15].clone().into() * main_current[16].clone().into()));
        builder.when_transition().assert_zero::<_>(main_next[6].clone().into() - (main_current[9].clone().into() + main_current[10].clone().into() + main_current[11].clone().into() + main_current[12].clone().into() + main_current[13].clone().into() * main_current[14].clone().into() * main_current[15].clone().into() * main_current[16].clone().into()));
        builder.when_transition().assert_zero::<_>(main_next[7].clone().into() - (main_current[9].clone().into() * main_current[13].clone().into() + main_current[10].clone().into() * main_current[14].clone().into() + main_current[11].clone().into() * main_current[15].clone().into() + main_current[12].clone().into() * main_current[16].clone().into() + (main_current[9].clone().into() + main_current[13].clone().into()) * (main_current[10].clone().into() + main_current[14].clone().into()) * (main_current[11].clone().into() + main_current[15].clone().into()) * (main_current[12].clone().into() + main_current[16].clone().into())));
        builder.when_transition().assert_zero::<_>(main_next[8].clone().into() - (main_current[1].clone().into() + main_current[9].clone().into() * main_current[13].clone().into() + main_current[10].clone().into() * main_current[14].clone().into() + main_current[11].clone().into() * main_current[15].clone().into() + main_current[12].clone().into() * main_current[16].clone().into() + main_current[9].clone().into() * main_current[13].clone().into() + main_current[10].clone().into() * main_current[14].clone().into() + main_current[11].clone().into() * main_current[15].clone().into() + main_current[12].clone().into() * main_current[16].clone().into()));
    }
}