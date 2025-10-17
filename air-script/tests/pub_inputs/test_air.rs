use winter_air::Air;
use winter_math::fields::f64::BaseElement as Felt;
use winterfell::{Trace, TraceTable};

use crate::{
    generate_air_test,
    helpers::{AirTester, MyTraceTable},
    pub_inputs::pub_inputs::PublicInputs,
};

#[derive(Clone)]
struct PubInputsAirTester {}

impl AirTester for PubInputsAirTester {
    type PubInputs = PublicInputs;

    fn build_main_trace(&self, length: usize) -> MyTraceTable {
        let trace_width = 4;
        let mut trace = TraceTable::new(trace_width, length);
        let start = Felt::new(0);

        trace.fill(
            |state| {
                state[0] = start;
                state[1] = start;
                state[2] = start;
                state[3] = start;
            },
            |_, state| {},
        );

        MyTraceTable::new(trace, 0)
    }

    fn public_inputs(&self) -> PublicInputs {
        let zero = Felt::new(0);
        PublicInputs::new([zero; 4], [zero; 4], [zero; 4], [zero; 20])
    }
}

generate_air_test!(
    test_pub_inputs_air,
    crate::pub_inputs::pub_inputs::PubInputsAir,
    PubInputsAirTester,
    1024
);
