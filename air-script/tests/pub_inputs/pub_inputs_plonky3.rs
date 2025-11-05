use p3_air::{Air, BaseAir, BaseAirWithPublicValues, ExtensionBuilder};
use p3_matrix::Matrix;
use p3_field::{Field, PrimeCharacteristicRing};
use crate::helpers::{AirScriptAir, AirScriptBuilder};

pub const NUM_COLUMNS: usize = 4;

pub const NUM_PUBLIC_VALUES: usize = 32;

pub struct PubInputsAir;

impl<F> BaseAir<F> for PubInputsAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for PubInputsAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: Field> AirScriptAir<F> for PubInputsAir {
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
        builder.when_first_row().assert_zero_ext::<_>(main_current[0].clone().into() - public_values[8].into());
        builder.when_first_row().assert_zero_ext::<_>(main_current[1].clone().into() - public_values[9].into());
        builder.when_first_row().assert_zero_ext::<_>(main_current[2].clone().into() - public_values[10].into());
        builder.when_first_row().assert_zero_ext::<_>(main_current[3].clone().into() - public_values[11].into());
        builder.when_last_row().assert_zero_ext::<_>(main_current[0].clone().into() - public_values[12].into());
        builder.when_last_row().assert_zero_ext::<_>(main_current[1].clone().into() - public_values[13].into());
        builder.when_last_row().assert_zero_ext::<_>(main_current[2].clone().into() - public_values[14].into());
        builder.when_last_row().assert_zero_ext::<_>(main_current[3].clone().into() - public_values[15].into());
        builder.when_transition().assert_zero_ext::<_>(main_next[0].clone().into() - (main_current[1].clone().into() + main_current[2].clone().into()));
    }
}

impl<AB: AirScriptBuilder> Air<AB> for PubInputsAir {
    fn eval(&self, builder: &mut AB) {
        <Self as AirScriptAir<AB::F>>::eval(self, builder);
    }
}