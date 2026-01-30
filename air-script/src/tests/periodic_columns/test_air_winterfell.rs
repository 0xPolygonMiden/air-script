use winter_air::Air;
use winter_math::fields::f64::BaseElement as Felt;
use winterfell::{Trace, TraceTable};

use crate::{
    generate_air_winterfell_test,
    test_utils::winterfell_traits::{AirTester, MyTraceTable},
    tests::periodic_columns::periodic_columns::PublicInputs,
};

#[derive(Clone)]
struct PeriodicColumnsAirTester {}

impl AirTester for PeriodicColumnsAirTester {
    type PubInputs = PublicInputs;

    fn build_main_trace(&self, length: usize) -> MyTraceTable {
        let trace_width = 3;
        let mut trace = TraceTable::new(trace_width, length);
        let start = Felt::new(0);

        trace.fill(
            |state| {
                state[0] = start;
                state[1] = start;
                state[2] = start;
            },
            |_, state| {},
        );

        MyTraceTable::new(trace, 0)
    }

    fn public_inputs(&self) -> PublicInputs {
        let zero = Felt::new(0);
        PublicInputs::new([zero; 16])
    }
}

generate_air_winterfell_test!(
    test_periodic_columns_air,
    crate::tests::periodic_columns::periodic_columns::PeriodicColumnsAir,
    PeriodicColumnsAirTester,
    1024
);
