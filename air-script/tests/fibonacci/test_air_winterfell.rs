#[allow(unused_imports)]
use winter_air::{Air, AuxRandElements};
use winter_math::fields::f64::BaseElement as Felt;
#[allow(unused_imports)]
use winterfell::{AuxTraceWithMetadata, Trace, TraceTable, matrix::ColMatrix};

use crate::{
    fibonacci::fibonacci::PublicInputs,
    generate_air_winterfell_test,
    helpers::{AirTester, MyTraceTable},
};

#[derive(Clone)]
struct FibonacciAirTester {}

impl AirTester for FibonacciAirTester {
    type PubInputs = PublicInputs;

    fn build_main_trace(&self, length: usize) -> MyTraceTable {
        let trace_width = 2;
        let mut trace = TraceTable::new(trace_width, length);
        let start_a = Felt::new(1);
        let start_b = Felt::new(1);

        trace.fill(
            |state| {
                state[0] = start_a;
                state[1] = start_b;
            },
            |_, state| {
                let cur_a = state[0];
                let cur_b = state[1];
                state[0] = cur_b;
                state[1] = cur_a + cur_b;
            },
        );

        MyTraceTable::new(trace, 0)
    }

    fn public_inputs(&self) -> PublicInputs {
        let one = Felt::new(1);
        let last = Felt::new(2178309); // 32nd Fibonacci number
        PublicInputs::new([one, one], [last])
    }
}

generate_air_winterfell_test!(
    test_fibonacci_air,
    crate::fibonacci::fibonacci::FibonacciAir,
    FibonacciAirTester,
    32
);
