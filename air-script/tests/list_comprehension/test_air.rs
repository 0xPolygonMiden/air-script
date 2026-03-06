use winter_air::Air;
use winter_math::fields::f64::BaseElement as Felt;
use winterfell::{Trace, TraceTable};

use crate::{
    generate_air_test,
    helpers::{AirTester, MyTraceTable},
    list_comprehension::list_comprehension::PublicInputs,
};

#[derive(Clone)]
struct ListComprehensionAirTester {}

impl AirTester for ListComprehensionAirTester {
    type PubInputs = PublicInputs;

    fn build_main_trace(&self, length: usize) -> MyTraceTable {
        let trace_width = 16;
        let mut trace = TraceTable::new(trace_width, length);
        let start = Felt::new(0);

        trace.fill(
            |state| {
                state[0] = start;
                state[1] = Felt::new(20);
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
                state[12] = start;
                state[13] = start;
                state[14] = Felt::new(10);
                state[15] = start;
            },
            |_, state| {
                state[3] = Felt::new(2);
            },
        );

        MyTraceTable::new(trace, 0)
    }

    fn public_inputs(&self) -> PublicInputs {
        let zero = Felt::new(0);
        PublicInputs::new([zero; 16])
    }
}

generate_air_test!(
    test_list_comprehension_air,
    crate::list_comprehension::list_comprehension::ListComprehensionAir,
    ListComprehensionAirTester,
    1024
);
