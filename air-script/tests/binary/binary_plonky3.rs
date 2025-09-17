use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir, BaseAirWithPublicValues};
use p3_matrix::Matrix;
use p3_field::PrimeCharacteristicRing;

pub const NUM_COLUMNS: usize = 2;

pub const NUM_PUBLIC_VALUES: usize = 16;

pub struct BinaryAir;

impl<F> BaseAir<F> for BinaryAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for BinaryAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<AB: AirBuilderWithPublicValues> Air<AB> for BinaryAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let public_values = builder.public_values().to_vec();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        builder.when_first_row().assert_zero::<_>(main_current[0] - public_values[0].into());
        builder.when_transition().assert_zero::<_>(main_current[0] * main_current[0] - main_current[0]);
        builder.when_transition().assert_zero::<_>(main_current[1] * main_current[1] - main_current[1]);
    }
}