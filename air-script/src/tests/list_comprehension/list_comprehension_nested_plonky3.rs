use p3_field::{ExtensionField, Field, PrimeCharacteristicRing};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrixView;
use p3_matrix::stack::VerticalPair;
use p3_miden_air::{MidenAir, MidenAirBuilder, RowMajorMatrix};

pub const MAIN_WIDTH: usize = 2;
pub const AUX_WIDTH: usize = 0;
pub const NUM_PERIODIC_VALUES: usize = 0;
pub const PERIOD: usize = 0;
pub const NUM_PUBLIC_VALUES: usize = 1;
pub const NUM_BETA_CHALLENGES: usize = 0;

pub struct ListComprehensionAir;

impl<F, EF> MidenAir<F, EF> for ListComprehensionAir
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
        builder.when_first_row().assert_zero(main_current[0].clone().into());

        // Main integrity/transition constraints
        builder.assert_zero(main_current[0].clone().into() + main_current[1].clone().into().double() - AB::Expr::from_u64(3));
        builder.assert_zero(main_current[0].clone().into().double() + main_current[1].clone().into() * AB::Expr::from_u64(3) - AB::Expr::from_u64(5));
        builder.assert_zero(main_current[0].clone().into() * AB::Expr::from_u64(3) + main_current[1].clone().into() * AB::Expr::from_u64(4) - AB::Expr::from_u64(7));

        // Aux boundary constraints

        // Aux integrity/transition constraints
    }
}