//! Cross-backend comparison test for the FunctionsComplex AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the FunctionsComplex AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::functions::{
        functions_complex::{FunctionsAir as WinterfellFunctionsComplexAir, PublicInputs},
        functions_complex_plonky3::FunctionsAir as Plonky3FunctionsComplexAir,
    },
};

/// Configuration for FunctionsComplex AIR cross-backend comparison tests.
struct FunctionsComplexTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl FunctionsComplexTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut trace = vec![vec![Felt::ZERO; length]; 17];

        for row in 1..length {
            trace[3][row] = Felt::new(2);
        }

        trace
    }
}

impl CrossBackendTestConfig for FunctionsComplexTestConfig {
    type WinterfellAir = WinterfellFunctionsComplexAir;
    type Plonky3Air = Plonky3FunctionsComplexAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        17
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
    ) -> WinterfellFunctionsComplexAir {
        WinterfellFunctionsComplexAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3FunctionsComplexAir {
        Plonky3FunctionsComplexAir
    }
}

#[test]
fn test_functions_complex_air_constraint_comparison() {
    let config = FunctionsComplexTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "FunctionsComplex AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_functions_complex_air_constraint_comparison_random_inputs() {
    let config = FunctionsComplexTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_functions_complex_air_constraint_comparison_random_inputs",
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

    println!("FunctionsComplex AIR random comparison passed for all 51 iterations (0-50)");
}
