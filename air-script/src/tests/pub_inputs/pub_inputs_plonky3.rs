use p3_field::{ExtensionField, Field, PrimeCharacteristicRing};
use p3_matrix::Matrix;
use p3_miden_air::{MidenAir, MidenAirBuilder};

pub const MAIN_WIDTH: usize = 4;
pub const AUX_WIDTH: usize = 0;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 32;
pub const NUM_BETA_CHALLENGES: usize = 0;

pub struct PubInputsAir;

impl<F, EF> MidenAir<F, EF> for PubInputsAir
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
        builder.when_first_row().assert_zero(main_current[0].clone().into() - public_values[8].into());
        builder.when_first_row().assert_zero(main_current[1].clone().into() - public_values[9].into());
        builder.when_first_row().assert_zero(main_current[2].clone().into() - public_values[10].into());
        builder.when_first_row().assert_zero(main_current[3].clone().into() - public_values[11].into());
        builder.when_last_row().assert_zero(main_current[0].clone().into() - public_values[12].into());
        builder.when_last_row().assert_zero(main_current[1].clone().into() - public_values[13].into());
        builder.when_last_row().assert_zero(main_current[2].clone().into() - public_values[14].into());
        builder.when_last_row().assert_zero(main_current[3].clone().into() - public_values[15].into());

        // Main integrity/transition constraints
        builder.when_transition().assert_zero(main_next[0].clone().into() - (main_current[1].clone().into() + main_current[2].clone().into()));

        // Aux boundary constraints

        // Aux integrity/transition constraints
    }
}