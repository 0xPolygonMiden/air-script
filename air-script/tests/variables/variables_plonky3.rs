use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir, BaseAirWithPublicValues};
use p3_matrix::Matrix;
use p3_field::PrimeCharacteristicRing;
use crate::helpers::{AirBuilderWithPeriodicColumns, BaseAirWithPeriodicColumns};

pub const NUM_COLUMNS: usize = 4;

pub const NUM_PUBLIC_VALUES: usize = 32;

pub struct VariablesAir;

impl<F> BaseAir<F> for VariablesAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for VariablesAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: PrimeCharacteristicRing> BaseAirWithPeriodicColumns<F> for VariablesAir {
    fn get_periodic_columns(&self) -> Vec<Vec<F>> {
        vec![
            vec![F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(0)],
        ]
    }
}

impl<AB: AirBuilderWithPublicValues + AirBuilderWithPeriodicColumns> Air<AB> for VariablesAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values = builder.periodic_columns();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        builder.when_first_row().assert_zero::<_>(main_current[1].into());
        builder.when_last_row().assert_zero::<_>(main_current[1].into() - AB::Expr::ONE);
        builder.assert_zero::<_>(main_current[0].into() * main_current[0].into() - main_current[0].into());
        builder.when_transition().assert_zero::<_>(periodic_values[0].into() * (main_next[0].into() - main_current[0].into()));
        builder.assert_zero::<_>((AB::Expr::ONE - main_current[0].into()) * (main_current[3].into() - main_current[1].into() - main_current[2].into()) - (AB::Expr::from_u64(6) - (AB::Expr::from_u64(7) - main_current[0].into())));
        builder.when_transition().assert_zero::<_>(main_current[0].into() * (main_current[3].into() - main_current[1].into() * main_current[2].into()) - (AB::Expr::ONE - main_next[0].into()));
    }
}