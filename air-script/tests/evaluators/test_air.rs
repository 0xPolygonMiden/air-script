use winter_air::Air;
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};
use winterfell::{Trace, TraceTable};

use crate::{
    evaluators::evaluators::PublicInputs,
    generate_air_test,
    helpers::{AirTester, MyTraceTable},
};

#[derive(Clone)]
struct EvaluatorsAirTester {}

impl AirTester for EvaluatorsAirTester {
    type PubInputs = PublicInputs;

    fn build_main_trace(&self, length: usize) -> MyTraceTable {
        let trace_width = 7;
        let mut trace = TraceTable::new(trace_width, length);
        let start = Felt::new(0);

        trace.fill(
            |state| {
                state[0] = start;
                state[1] = start;
                state[2] = start;
                state[3] = start;
                state[4] = Felt::ZERO;
                state[5] = Felt::new(1);
                state[6] = Felt::new(4);
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
    test_evaluators_air,
    crate::evaluators::evaluators::EvaluatorsAir,
    EvaluatorsAirTester,
    1024
);
