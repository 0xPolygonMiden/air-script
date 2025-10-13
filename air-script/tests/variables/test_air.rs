use winter_air::Air;
use winter_math::fields::f64::BaseElement as Felt;
use winterfell::{Trace, TraceTable};

use crate::{
    generate_air_test,
    helpers::{AirTester, MyTraceTable},
    variables::variables::PublicInputs,
};

#[derive(Clone)]
struct VariablesAirTester {}

impl AirTester for VariablesAirTester {
    type PubInputs = PublicInputs;

    fn build_main_trace(&self, length: usize) -> MyTraceTable {
        let trace_width = 4;
        let mut trace = TraceTable::new(trace_width, length);
        let start = Felt::new(0);

        trace.fill(
            |state| {
                state[0] = Felt::new(1);
                state[1] = start;
                state[2] = start;
                state[3] = start;
            },
            |_, state| {
                state[1] = Felt::new(1);
            },
        );

        MyTraceTable::new(trace, 0)
    }

    fn public_inputs(&self) -> PublicInputs {
        let zero = Felt::new(0);
        PublicInputs::new([zero; 16], [zero; 16])
    }
}

generate_air_test!(
    test_variables_air,
    crate::variables::variables::VariablesAir,
    VariablesAirTester,
    1024
);
