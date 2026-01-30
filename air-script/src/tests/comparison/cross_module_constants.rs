//! Cross-backend comparison test for the CrossModuleConstants AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the CrossModuleConstants AIR at every row of the trace.
//!
//! The CrossModuleConstants AIR tests cross-module evaluators using constants:
//! - Boundary: `a.first = 0`
//! - Transition: `result = a + 2*b + 3*c + 4*d`
//!
//! Trace columns: [a, b, c, d, result] (5 total)
//! Column indices: a=0, b=1, c=2, d=3, result=4

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::cross_module_constants::{
        cross_mod_constants::{
            CrossModuleConstantsTest as WinterfellCrossModuleConstantsTest, PublicInputs,
        },
        cross_mod_constants_plonky3::CrossModuleConstantsTest as Plonky3CrossModuleConstantsTest,
    },
};

/// Configuration for CrossModuleConstants AIR cross-backend comparison tests.
struct CrossModuleConstantsTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl CrossModuleConstantsTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut trace = vec![vec![Felt::ZERO; length]; 5];

        for row in 0..length {
            let a = if row == 0 { Felt::ZERO } else { Felt::new(row as u64) };
            let b = Felt::new((row + 1) as u64);
            let c = Felt::new((row + 2) as u64);
            let d = Felt::new((row + 3) as u64);
            let result = a + b.double() + c * Felt::new(3) + d * Felt::new(4);

            trace[0][row] = a;
            trace[1][row] = b;
            trace[2][row] = c;
            trace[3][row] = d;
            trace[4][row] = result;
        }

        trace
    }

    fn expected_value(&self) -> Felt {
        let trace = self.build_trace();
        let last_step = self.trace_length - 2;
        trace[4][last_step]
    }
}

impl CrossBackendTestConfig for CrossModuleConstantsTestConfig {
    type WinterfellAir = WinterfellCrossModuleConstantsTest;
    type Plonky3Air = Plonky3CrossModuleConstantsTest;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        5
    }

    fn trace_length(&self) -> usize {
        self.trace_length
    }

    fn build_winterfell_trace(&self) -> Vec<Vec<Felt>> {
        self.build_trace()
    }

    fn build_winterfell_public_inputs(&self) -> PublicInputs {
        PublicInputs::new([self.expected_value()])
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellCrossModuleConstantsTest {
        WinterfellCrossModuleConstantsTest::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3CrossModuleConstantsTest {
        Plonky3CrossModuleConstantsTest
    }
}

#[test]
fn test_cross_module_constants_air_constraint_comparison() {
    let config = CrossModuleConstantsTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "CrossModuleConstants AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_cross_module_constants_air_constraint_comparison_random_inputs() {
    let config = CrossModuleConstantsTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_cross_module_constants_air_constraint_comparison_random_inputs",
                iteration,
            },
        );

        if !result.is_ok() {
            panic!(
                "Random constraint evaluation comparison failed (iteration={})!\n\n{}",
                iteration,
                result.format_report()
            );
        }
    }

    println!("CrossModuleConstants AIR random comparison passed for all 51 iterations (0-50)");
}
