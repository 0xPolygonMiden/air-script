use winter_air::Air;
use winter_math::fields::f64::BaseElement as Felt;
use winterfell::{Trace, TraceTable};

use crate::{
    constants::constants::PublicInputs,
    generate_air_test,
    helpers::{AirTester, MyTraceTable},
};

#[derive(Clone)]
struct ConstantsAirTester {}

impl AirTester for ConstantsAirTester {
    type PubInputs = PublicInputs;

    fn build_main_trace(&self, length: usize) -> MyTraceTable {
        let trace_width = 7;
        let mut trace = TraceTable::new(trace_width, length);

        trace.fill(
            |state| {
                state[0] = Felt::new(1);
                state[1] = Felt::new(1);
                state[2] = Felt::new(0);
                state[3] = Felt::new(1);
                state[4] = Felt::new(1);
                state[5] = Felt::new(0);
                state[6] = Felt::new(0);
            },
            |_, state| {
                state[0] += Felt::new(1);
                state[1] = Felt::new(0);
                state[5] += Felt::new(1);
            },
        );

        MyTraceTable::new(trace, 0)
    }

    fn public_inputs(&self) -> PublicInputs {
        let zero = Felt::new(0);
        PublicInputs::new([zero; 4], [zero; 4], [zero; 4], [zero; 20])
    }
}

generate_air_test!(
    test_constants_air,
    crate::constants::constants::ConstantsAir,
    ConstantsAirTester,
    1024
);
