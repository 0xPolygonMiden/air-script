use p3_field::{ExtensionField, Field, PrimeCharacteristicRing};
use p3_matrix::Matrix;
use p3_miden_air::{MidenAir, MidenAirBuilder};
use crate::test_utils::plonky3_traits::AirScriptAir;

pub const MAIN_WIDTH: usize = 9;
pub const AUX_WIDTH: usize = 0;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 16;
pub const NUM_BETA_CHALLENGES: usize = 0;

pub struct FunctionsAir;

impl<F, EF> MidenAir<F, EF> for FunctionsAir
where F: Field,
      EF: ExtensionField<F>,
{
    fn width(&self) -> usize {
        MAIN_WIDTH
    }

    fn eval<AB>(&self, builder: &mut AB)
    where AB: MidenAirBuilder<F = F, EF = EF>,
    {
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let preprocessed = builder.preprocessed();
        let periodic_values = preprocessed.row_slice(0).unwrap();
        let main = builder.main();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );

        // Main boundary constraints
        builder.when_first_row().assert_zero(main_current[3].clone().into());

        // Main integrity/transition constraints
        builder.assert_zero(main_current[0].clone().into() * main_current[3].clone().into() - AB::Expr::ONE);
        builder.assert_zero(main_current[4].clone().into() * main_current[5].clone().into() * main_current[6].clone().into() * main_current[7].clone().into() * main_current[3].clone().into() - AB::Expr::ONE);
        builder.assert_zero((main_current[4].clone().into() + main_current[5].clone().into() + main_current[6].clone().into() + main_current[7].clone().into()) * main_current[4].clone().into() * main_current[5].clone().into() * main_current[6].clone().into() * main_current[7].clone().into() - AB::Expr::ONE);
        builder.assert_zero(main_current[4].clone().into() * main_current[5].clone().into() * main_current[6].clone().into() * main_current[7].clone().into() - AB::Expr::ONE);
        builder.assert_zero(main_current[0].clone().into() * main_current[4].clone().into() * main_current[5].clone().into() * main_current[6].clone().into() * main_current[7].clone().into() - AB::Expr::ONE);
        builder.assert_zero(main_current[1].clone().into() + (main_current[4].clone().into() + main_current[5].clone().into() + main_current[6].clone().into() + main_current[7].clone().into()) * main_current[4].clone().into() * main_current[5].clone().into() * main_current[6].clone().into() * main_current[7].clone().into() - AB::Expr::ONE);
        builder.assert_zero(main_current[4].clone().into() + main_current[5].clone().into() + main_current[6].clone().into() + main_current[7].clone().into() - AB::Expr::ONE);
        builder.assert_zero((main_current[4].clone().into() + main_current[5].clone().into() + main_current[6].clone().into() + main_current[7].clone().into()) * AB::Expr::from_u64(4) - AB::Expr::ONE);

        // Aux boundary constraints

        // Aux integrity/transition constraints
    }
}

impl<F: Field, EF: ExtensionField<F>> AirScriptAir<F, EF> for FunctionsAir {
    fn num_beta_challenges(&self) -> usize {
        NUM_BETA_CHALLENGES
    }

    fn periodic_table(&self) -> Vec<Vec<F>> {
        vec![]
    }
}