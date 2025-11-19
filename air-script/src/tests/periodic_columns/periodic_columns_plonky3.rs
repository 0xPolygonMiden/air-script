use p3_field::{ExtensionField, Field, PrimeCharacteristicRing};
use p3_matrix::Matrix;
use p3_miden_air::{MidenAir, MidenAirBuilder};
use crate::test_utils::plonky3_traits::AirScriptAir;

pub const MAIN_WIDTH: usize = 3;
pub const AUX_WIDTH: usize = 0;
pub const NUM_PERIODIC_VALUES: usize = 2;
pub const PERIOD: usize = 8;
pub const NUM_PUBLIC_VALUES: usize = 16;
pub const NUM_BETA_CHALLENGES: usize = 0;

pub struct PeriodicColumnsAir;

impl<F, EF> MidenAir<F, EF> for PeriodicColumnsAir
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
        builder.assert_zero_ext(AB::ExprEF::from(periodic_values[0].clone().into()) * (AB::ExprEF::from(main_current[1].clone().into()) + AB::ExprEF::from(main_current[2].clone().into())));
        builder.when_transition().assert_zero_ext(AB::ExprEF::from(periodic_values[1].clone().into()) * (AB::ExprEF::from(main_next[0].clone().into()) - AB::ExprEF::from(main_current[0].clone().into())));

        // Aux boundary constraints

        // Aux integrity/transition constraints
    }
}

impl<F: Field, EF: ExtensionField<F>> AirScriptAir<F, EF> for PeriodicColumnsAir {
    fn num_beta_challenges(&self) -> usize {
        NUM_BETA_CHALLENGES
    }

    fn periodic_table(&self) -> Vec<Vec<F>> {
        vec![
            vec![F::from_u64(1), F::from_u64(0), F::from_u64(0), F::from_u64(0)],
            vec![F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(1), F::from_u64(0)],
        ]
    }
}