use winter_air::Air;
use winter_math::fields::f64::BaseElement as Felt;
use winterfell::{Trace, TraceTable};

use crate::{
    generate_air_winterfell_test,
    helpers::{AirTester, MyTraceTable},
    list_folding::list_folding::PublicInputs,
};

#[derive(Clone)]
struct ListFoldingAirTester {}

impl AirTester for ListFoldingAirTester {
    type PubInputs = PublicInputs;

    fn build_main_trace(&self, length: usize) -> MyTraceTable {
        let trace_width = 17;
        let mut trace = TraceTable::new(trace_width, length);
        let start = Felt::new(0);

        trace.fill(
            |state| {
                state[0] = start;
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
                state[12] = start;
                state[13] = start;
                state[14] = start;
                state[15] = start;
                state[16] = start;
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

generate_air_winterfell_test!(
    test_list_folding_air,
    crate::list_folding::list_folding::ListFoldingAir,
    ListFoldingAirTester,
    1024
);
