use winter_air::Air;
use winter_math::fields::f64::BaseElement as Felt;
use winterfell::{Trace, TraceTable};

use crate::{
    generate_air_winterfell_test,
    test_utils::winterfell_traits::{AirTester, MyTraceTable},
    tests::computed_indices::computed_indices_simple::PublicInputs,
};

#[derive(Clone)]
struct ComputedIndicesAirTester {}

impl AirTester for ComputedIndicesAirTester {
    type PubInputs = PublicInputs;

    fn build_main_trace(&self, length: usize) -> MyTraceTable {
        let trace_width = 8;
        let mut trace = TraceTable::new(trace_width, length);

        trace.fill(
            |state| {
                state[0] = Felt::new(0);
                state[1] = Felt::new(2);
                state[2] = Felt::new(4);
                state[3] = Felt::new(6);
                state[4] = Felt::new(0);
                state[5] = Felt::new(0);
                state[6] = Felt::new(0);
                state[7] = Felt::new(0);
            },
            |_, state| {
                state[4] *= Felt::new(0);
                state[5] *= Felt::new(2);
                state[6] *= Felt::new(6);
                state[7] *= Felt::new(12);
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
    test_computed_indices_air,
    crate::tests::computed_indices::computed_indices_simple::ComputedIndicesAir,
    ComputedIndicesAirTester,
    1024
);
