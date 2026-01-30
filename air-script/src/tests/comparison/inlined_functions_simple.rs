//! Cross-backend comparison test for the InlinedFunctionsSimple AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the inlined FunctionsSimple AIR at every row of the trace.
//!
//! NOTE: The inlined variant shares generated outputs with `functions_simple`.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::functions::{
        functions_simple::{FunctionsAir as WinterfellFunctionsSimpleAir, PublicInputs},
        functions_simple_plonky3::FunctionsAir as Plonky3FunctionsSimpleAir,
    },
};

/// Configuration for inlined FunctionsSimple AIR cross-backend comparison tests.
struct InlinedFunctionsSimpleTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl InlinedFunctionsSimpleTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;

        vec![vec![Felt::ZERO; length]; 9]
    }
}

impl CrossBackendTestConfig for InlinedFunctionsSimpleTestConfig {
    type WinterfellAir = WinterfellFunctionsSimpleAir;
    type Plonky3Air = Plonky3FunctionsSimpleAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        9
    }

    fn trace_length(&self) -> usize {
        self.trace_length
    }

    fn build_winterfell_trace(&self) -> Vec<Vec<Felt>> {
        self.build_trace()
    }

    fn build_winterfell_public_inputs(&self) -> PublicInputs {
        PublicInputs::new([Felt::ZERO; 16])
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellFunctionsSimpleAir {
        WinterfellFunctionsSimpleAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3FunctionsSimpleAir {
        Plonky3FunctionsSimpleAir
    }
}

#[test]
fn test_inlined_functions_simple_air_constraint_comparison() {
    let config = InlinedFunctionsSimpleTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "InlinedFunctionsSimple AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_inlined_functions_simple_air_constraint_comparison_random_inputs() {
    let config = InlinedFunctionsSimpleTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_inlined_functions_simple_air_constraint_comparison_random_inputs",
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

    println!("InlinedFunctionsSimple AIR random comparison passed for all 51 iterations (0-50)");
}
