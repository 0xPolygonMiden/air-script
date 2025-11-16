use winter_air::Air;
use winter_math::fields::f64::BaseElement as Felt;
use winterfell::{Trace, TraceTable};

use crate::{
    binary::binary::PublicInputs,
    generate_air_test,
    helpers::{AirTester, MyTraceTable},
};

#[derive(Clone)]
struct BinaryAirTester {}

impl AirTester for BinaryAirTester {
    type PubInputs = PublicInputs;

    fn build_main_trace(&self, length: usize) -> MyTraceTable {
        let trace_width = 2;
        let mut trace = TraceTable::new(trace_width, length);
        let start = Felt::new(0);

        trace.fill(
            |state| {
                state[0] = start;
                state[1] = start;
            },
            |_, state| {
                state[0] = Felt::new(1) - state[0];
                state[1] = Felt::new(1) - state[1];
            },
        );

        MyTraceTable::new(trace, 0)
    }

    fn public_inputs(&self) -> PublicInputs {
        let zero = Felt::new(0);
        PublicInputs::new([zero; 16])
    }
}

generate_air_test!(test_binary_air, crate::binary::binary::BinaryAir, BinaryAirTester, 1024);
