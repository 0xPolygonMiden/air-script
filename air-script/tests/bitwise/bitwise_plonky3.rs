use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir, BaseAirWithPublicValues};
use p3_matrix::Matrix;
use p3_field::PrimeCharacteristicRing;
use crate::helpers::{AirBuilderWithPeriodicColumns, BaseAirWithPeriodicColumns};

pub const NUM_COLUMNS: usize = 14;

pub const NUM_PUBLIC_VALUES: usize = 16;

pub struct BitwiseAir;

impl<F> BaseAir<F> for BitwiseAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for BitwiseAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: PrimeCharacteristicRing> BaseAirWithPeriodicColumns<F> for BitwiseAir {
    fn get_periodic_columns(&self) -> Vec<Vec<F>> {
        vec![
            vec![F::from_u64(1), F::from_u64(0), F::from_u64(0), F::from_u64(0), F::from_u64(0), F::from_u64(0), F::from_u64(0), F::from_u64(0)],
            vec![F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(0)],
        ]
    }
}

impl<AB: AirBuilderWithPublicValues + AirBuilderWithPeriodicColumns> Air<AB> for BitwiseAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values = builder.periodic_columns();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        builder.when_first_row().assert_zero::<_>(main_current[13].into());
        builder.assert_zero::<_>(main_current[0].into() * main_current[0].into() - main_current[0].into());
        builder.when_transition().assert_zero::<_>(periodic_values[1].into() * (main_next[0].into() - main_current[0].into()));
        builder.assert_zero::<_>(main_current[3].into() * main_current[3].into() - main_current[3].into());
        builder.assert_zero::<_>(main_current[4].into() * main_current[4].into() - main_current[4].into());
        builder.assert_zero::<_>(main_current[5].into() * main_current[5].into() - main_current[5].into());
        builder.assert_zero::<_>(main_current[6].into() * main_current[6].into() - main_current[6].into());
        builder.assert_zero::<_>(main_current[7].into() * main_current[7].into() - main_current[7].into());
        builder.assert_zero::<_>(main_current[8].into() * main_current[8].into() - main_current[8].into());
        builder.assert_zero::<_>(main_current[9].into() * main_current[9].into() - main_current[9].into());
        builder.assert_zero::<_>(main_current[10].into() * main_current[10].into() - main_current[10].into());
        builder.assert_zero::<_>(periodic_values[0].into() * (main_current[1].into() - (main_current[3].into() + main_current[4].into().double() + AB::Expr::from_u64(4) * main_current[5].into() + AB::Expr::from_u64(8) * main_current[6].into())));
        builder.assert_zero::<_>(periodic_values[0].into() * (main_current[2].into() - (main_current[7].into() + main_current[8].into().double() + AB::Expr::from_u64(4) * main_current[9].into() + AB::Expr::from_u64(8) * main_current[10].into())));
        builder.when_transition().assert_zero::<_>(periodic_values[1].into() * (main_next[1].into() - (main_current[1].into() * AB::Expr::from_u64(16) + main_current[3].into() + main_current[4].into().double() + AB::Expr::from_u64(4) * main_current[5].into() + AB::Expr::from_u64(8) * main_current[6].into())));
        builder.when_transition().assert_zero::<_>(periodic_values[1].into() * (main_next[2].into() - (main_current[2].into() * AB::Expr::from_u64(16) + main_current[7].into() + main_current[8].into().double() + AB::Expr::from_u64(4) * main_current[9].into() + AB::Expr::from_u64(8) * main_current[10].into())));
        builder.assert_zero::<_>(periodic_values[0].into() * main_current[11].into());
        builder.when_transition().assert_zero::<_>(periodic_values[1].into() * (main_current[12].into() - main_next[11].into()));
        builder.assert_zero::<_>((AB::Expr::ONE - main_current[0].into()) * (main_current[12].into() - (main_current[11].into() * AB::Expr::from_u64(16) + main_current[3].into() * main_current[7].into() + main_current[4].into().double() * main_current[8].into() + AB::Expr::from_u64(4) * main_current[5].into() * main_current[9].into() + AB::Expr::from_u64(8) * main_current[6].into() * main_current[10].into())) + main_current[0].into() * (main_current[12].into() - (main_current[11].into() * AB::Expr::from_u64(16) + main_current[3].into() + main_current[7].into() - main_current[3].into().double() * main_current[7].into() + (main_current[4].into() + main_current[8].into() - main_current[4].into().double() * main_current[8].into()).double() + AB::Expr::from_u64(4) * (main_current[5].into() + main_current[9].into() - main_current[5].into().double() * main_current[9].into()) + AB::Expr::from_u64(8) * (main_current[6].into() + main_current[10].into() - main_current[6].into().double() * main_current[10].into()))));
    }
}