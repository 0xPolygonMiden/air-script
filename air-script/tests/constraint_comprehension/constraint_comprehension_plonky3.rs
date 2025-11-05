use p3_air::{Air, BaseAir, BaseAirWithPublicValues, ExtensionBuilder};
use p3_matrix::Matrix;
use p3_field::{Field, PrimeCharacteristicRing};
use crate::helpers::{AirScriptAir, AirScriptBuilder};

pub const NUM_COLUMNS: usize = 14;

pub const NUM_PUBLIC_VALUES: usize = 16;

pub struct ConstraintComprehensionAir;

impl<F> BaseAir<F> for ConstraintComprehensionAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for ConstraintComprehensionAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: Field> AirScriptAir<F> for ConstraintComprehensionAir {
     const MAIN_WIDTH: usize = NUM_COLUMNS;
     const AUX_WIDTH: usize = 0;
     const PERIOD: usize = 0;
     const NUM_ALPHA_CHALLENGES: usize = 0;
    fn periodic_table(&self) -> Vec<Vec<F>> {
        vec![
        ]
    }

    fn eval<AB>(&self, builder: &mut AB)
    where AB: AirScriptBuilder<F = F>,
    {
        let main = builder.main();
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values = builder.periodic_evals().to_vec();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );
        builder.when_first_row().assert_zero_ext::<_>(main_current[8].clone().into());
        builder.assert_zero_ext::<_>(main_current[6].clone().into() - main_current[10].clone().into());
        builder.assert_zero_ext::<_>(main_current[7].clone().into() - main_current[11].clone().into());
        builder.assert_zero_ext::<_>(main_current[8].clone().into() - main_current[12].clone().into());
        builder.assert_zero_ext::<_>(main_current[9].clone().into() - main_current[13].clone().into());
    }
}

impl<AB: AirScriptBuilder> Air<AB> for ConstraintComprehensionAir {
    fn eval(&self, builder: &mut AB) {
        <Self as AirScriptAir<AB::F>>::eval(self, builder);
    }
}