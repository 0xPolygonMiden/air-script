use p3_field::{ExtensionField, Field, PrimeCharacteristicRing};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrixView;
use p3_matrix::stack::VerticalPair;
use p3_miden_air::{BusType, MidenAir, MidenAirBuilder, RowMajorMatrix};

pub const MAIN_WIDTH: usize = 2;
pub const AUX_WIDTH: usize = 0;
pub const NUM_PERIODIC_VALUES: usize = 2;
pub const PERIOD: usize = 2;
pub const NUM_PUBLIC_VALUES: usize = 1;
pub const MAX_BETA_CHALLENGE_POWER: usize = 0;

pub struct ComprehensionPeriodicBindingTest;

impl<F, EF> MidenAir<F, EF> for ComprehensionPeriodicBindingTest
where F: Field,
      EF: ExtensionField<F>,
{
    fn width(&self) -> usize {
        MAIN_WIDTH
    }

    fn num_public_values(&self) -> usize {
        NUM_PUBLIC_VALUES
    }

    fn periodic_table(&self) -> Vec<Vec<F>> {
        vec![
            vec![F::from_u64(1), F::from_u64(2)],
            vec![F::from_u64(3), F::from_u64(4)],
        ]
    }

    fn eval<AB>(&self, builder: &mut AB)
    where AB: MidenAirBuilder<F = F>,
    {
        let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect("Wrong number of public values");
        let periodic_values: [_; NUM_PERIODIC_VALUES] = builder.periodic_evals().try_into().expect("Wrong number of periodic values");
        // Note: for now, we do not have any preprocessed values
        // let preprocessed = builder.preprocessed();
        let main = builder.main();
        let (main_current, main_next) = (
            main.row_slice(0).unwrap(),
            main.row_slice(1).unwrap(),
        );

        // Main boundary constraints
        builder.when_first_row().assert_zero(main_current[0].clone().into());

        // Main integrity/transition constraints
        builder.when_transition().assert_zero_ext(AB::ExprEF::from(main_next[0].clone().into()) - (AB::ExprEF::from(main_current[0].clone().into()) * AB::ExprEF::from(periodic_values[0].clone().into()) + AB::ExprEF::from(main_current[1].clone().into()) * AB::ExprEF::from(periodic_values[1].clone().into())));

        // Aux integrity/transition constraints
    }
}