use p3_air::{Air, BaseAir, BaseAirWithPublicValues, ExtensionBuilder};
use p3_matrix::Matrix;
use p3_field::{Field, PrimeCharacteristicRing};
use crate::helpers::{AirScriptAir, AirScriptBuilder};

pub const NUM_COLUMNS: usize = 6;

pub const NUM_PUBLIC_VALUES: usize = 1;

pub struct SelectorsAir;

impl<F> BaseAir<F> for SelectorsAir {
    fn width(&self) -> usize {
        NUM_COLUMNS
    }
}

impl<F> BaseAirWithPublicValues<F> for SelectorsAir {
    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }
}

impl<F: Field> AirScriptAir<F> for SelectorsAir {
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
        builder.when_first_row().assert_zero_ext::<_>(main_current[5].clone().into());
        builder.assert_zero_ext::<_>((main_current[0].clone().into() + (AB::Expr::ONE - main_current[0].clone().into()) * main_current[1].clone().into()) * main_current[3].clone().into() + (AB::Expr::ONE - main_current[0].clone().into()) * (AB::Expr::ONE - main_current[1].clone().into()) * (main_current[4].clone().into() - AB::Expr::from_u64(8)));
        builder.assert_zero_ext::<_>((AB::Expr::ONE - main_current[0].clone().into()) * (main_current[5].clone().into() - AB::Expr::from_u64(8)) + main_current[0].clone().into() * (main_current[4].clone().into() - AB::Expr::from_u64(2)));
        builder.assert_zero_ext::<_>(main_current[0].clone().into() * (main_current[5].clone().into() - AB::Expr::from_u64(4)) + (AB::Expr::ONE - main_current[0].clone().into()) * main_current[1].clone().into() * (main_current[4].clone().into() - AB::Expr::from_u64(6)));
    }
}

impl<AB: AirScriptBuilder> Air<AB> for SelectorsAir {
    fn eval(&self, builder: &mut AB) {
        <Self as AirScriptAir<AB::F>>::eval(self, builder);
    }
}