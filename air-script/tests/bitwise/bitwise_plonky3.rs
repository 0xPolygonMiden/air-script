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
        builder.when_first_row().assert_zero::<_>(main_current[13].clone().into());
        builder.assert_zero::<_>(main_current[0].clone().into() * main_current[0].clone().into() - main_current[0].clone().into());
        builder.when_transition().assert_zero::<_>(periodic_values[1].into() * (main_next[0].clone().into() - main_current[0].clone().into()));
        builder.assert_zero::<_>(main_current[3].clone().into() * main_current[3].clone().into() - main_current[3].clone().into());
        builder.assert_zero::<_>(main_current[4].clone().into() * main_current[4].clone().into() - main_current[4].clone().into());
        builder.assert_zero::<_>(main_current[5].clone().into() * main_current[5].clone().into() - main_current[5].clone().into());
        builder.assert_zero::<_>(main_current[6].clone().into() * main_current[6].clone().into() - main_current[6].clone().into());
        builder.assert_zero::<_>(main_current[7].clone().into() * main_current[7].clone().into() - main_current[7].clone().into());
        builder.assert_zero::<_>(main_current[8].clone().into() * main_current[8].clone().into() - main_current[8].clone().into());
        builder.assert_zero::<_>(main_current[9].clone().into() * main_current[9].clone().into() - main_current[9].clone().into());
        builder.assert_zero::<_>(main_current[10].clone().into() * main_current[10].clone().into() - main_current[10].clone().into());
        builder.assert_zero::<_>(periodic_values[0].into() * (main_current[1].clone().into() - (main_current[3].clone().into() + main_current[4].clone().into().double() + AB::Expr::from_u64(4) * main_current[5].clone().into() + AB::Expr::from_u64(8) * main_current[6].clone().into())));
        builder.assert_zero::<_>(periodic_values[0].into() * (main_current[2].clone().into() - (main_current[7].clone().into() + main_current[8].clone().into().double() + AB::Expr::from_u64(4) * main_current[9].clone().into() + AB::Expr::from_u64(8) * main_current[10].clone().into())));
        builder.when_transition().assert_zero::<_>(periodic_values[1].into() * (main_next[1].clone().into() - (main_current[1].clone().into() * AB::Expr::from_u64(16) + main_current[3].clone().into() + main_current[4].clone().into().double() + AB::Expr::from_u64(4) * main_current[5].clone().into() + AB::Expr::from_u64(8) * main_current[6].clone().into())));
        builder.when_transition().assert_zero::<_>(periodic_values[1].into() * (main_next[2].clone().into() - (main_current[2].clone().into() * AB::Expr::from_u64(16) + main_current[7].clone().into() + main_current[8].clone().into().double() + AB::Expr::from_u64(4) * main_current[9].clone().into() + AB::Expr::from_u64(8) * main_current[10].clone().into())));
        builder.assert_zero::<_>(periodic_values[0].into() * main_current[11].clone().into());
        builder.when_transition().assert_zero::<_>(periodic_values[1].into() * (main_current[12].clone().into() - main_next[11].clone().into()));
        builder.assert_zero::<_>((AB::Expr::ONE - main_current[0].clone().into()) * (main_current[12].clone().into() - (main_current[11].clone().into() * AB::Expr::from_u64(16) + main_current[3].clone().into() * main_current[7].clone().into() + main_current[4].clone().into().double() * main_current[8].clone().into() + AB::Expr::from_u64(4) * main_current[5].clone().into() * main_current[9].clone().into() + AB::Expr::from_u64(8) * main_current[6].clone().into() * main_current[10].clone().into())) + main_current[0].clone().into() * (main_current[12].clone().into() - (main_current[11].clone().into() * AB::Expr::from_u64(16) + main_current[3].clone().into() + main_current[7].clone().into() - main_current[3].clone().into().double() * main_current[7].clone().into() + (main_current[4].clone().into() + main_current[8].clone().into() - main_current[4].clone().into().double() * main_current[8].clone().into()).double() + AB::Expr::from_u64(4) * (main_current[5].clone().into() + main_current[9].clone().into() - main_current[5].clone().into().double() * main_current[9].clone().into()) + AB::Expr::from_u64(8) * (main_current[6].clone().into() + main_current[10].clone().into() - main_current[6].clone().into().double() * main_current[10].clone().into()))));
    }
}