use p3_field::{ExtensionField, Field, PrimeCharacteristicRing};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrixView;
use p3_miden_air::{MidenAir, MidenAirBuilder, RowMajorMatrix};

pub const MAIN_WIDTH: usize = 14;
pub const AUX_WIDTH: usize = 0;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 16;
pub const NUM_BETA_CHALLENGES: usize = 0;

pub struct ConstraintComprehensionAir;

impl<F, EF> MidenAir<F, EF> for ConstraintComprehensionAir
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
        let periodic_values: [_; NUM_PERIODIC_VALUES] = builder.periodic_evals().try_into().expect("Wrong number of periodic values");
        let preprocessed = builder.preprocessed();
        let main = builder.main();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );

        // Main boundary constraints
        builder.when_first_row().assert_zero(main_current[8].clone().into());

        // Main integrity/transition constraints
        builder.assert_zero(main_current[6].clone().into() - main_current[10].clone().into());
        builder.assert_zero(main_current[7].clone().into() - main_current[11].clone().into());
        builder.assert_zero(main_current[8].clone().into() - main_current[12].clone().into());
        builder.assert_zero(main_current[9].clone().into() - main_current[13].clone().into());

        // Aux boundary constraints

        // Aux integrity/transition constraints
    }
}