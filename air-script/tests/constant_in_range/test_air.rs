use winter_air::Air;
use winter_math::fields::f64::BaseElement as Felt;
use winterfell::{Trace, TraceTable};

use crate::{
    constant_in_range::constant_in_range::PublicInputs,
    generate_air_test,
    helpers::{AirTester, MyTraceTable},
};

#[derive(Clone)]
struct ConstantInRangeAirTester {}

impl AirTester for ConstantInRangeAirTester {
    type PubInputs = PublicInputs;

    fn build_main_trace(&self, length: usize) -> MyTraceTable {
        let trace_width = 12;
        let mut trace = TraceTable::new(trace_width, length);
        let start = Felt::new(0);

        trace.fill(
            |state| {
                state[0] = Felt::new(3);
                state[1] = start;
                state[2] = start;
                state[3] = start;
                state[4] = start;
                state[5] = start;
                state[6] = start;
                state[7] = start;
                state[8] = start;
                state[9] = start;
                state[10] = start;
                state[11] = start;
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

generate_air_test!(
    test_constant_in_range_air,
    crate::constant_in_range::constant_in_range::ConstantInRangeAir,
    ConstantInRangeAirTester,
    1024
);
