use p3_field::{ExtensionField, Field, PrimeCharacteristicRing};
use p3_matrix::Matrix;
use p3_miden_air::{MidenAir, MidenAirBuilder};

pub const MAIN_WIDTH: usize = 8;
pub const AUX_WIDTH: usize = 0;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 16;
pub const NUM_BETA_CHALLENGES: usize = 0;

pub struct ComputedIndicesAir;

impl<F, EF> MidenAir<F, EF> for ComputedIndicesAir
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
        builder.when_first_row().assert_zero(main_current[0].clone().into());

        // Main integrity/transition constraints
        builder.assert_zero(main_current[0].clone().into());
        builder.assert_zero(main_current[1].clone().into() - AB::Expr::from_u64(2));
        builder.assert_zero(main_current[2].clone().into() - AB::Expr::from_u64(4));
        builder.assert_zero(main_current[3].clone().into() - AB::Expr::from_u64(6));
        builder.when_transition().assert_zero(main_next[4].clone().into());
        builder.when_transition().assert_zero(main_next[5].clone().into() - main_current[5].clone().into().double());
        builder.when_transition().assert_zero(main_next[6].clone().into() - AB::Expr::from_u64(6) * main_current[6].clone().into());
        builder.when_transition().assert_zero(main_next[7].clone().into() - AB::Expr::from_u64(12) * main_current[7].clone().into());

        // Aux boundary constraints

        // Aux integrity/transition constraints
    }
}