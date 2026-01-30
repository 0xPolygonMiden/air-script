//! Cross-backend comparison test for the BusesComplex AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the BusesComplex AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::buses::{
        buses_complex::{BusesAir as WinterfellBusesComplexAir, PublicInputs},
        buses_complex_plonky3::BusesAir as Plonky3BusesComplexAir,
    },
};

/// Configuration for BusesComplex AIR cross-backend comparison tests.
struct BusesComplexTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl BusesComplexTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut trace = vec![vec![Felt::ZERO; length]; 7];

        for row in 1..length {
            trace[0][row] = Felt::ONE - trace[0][row - 1];
            trace[1][row] = Felt::ONE - trace[1][row - 1];
            trace[2][row] = Felt::ONE - trace[2][row - 1];
            trace[3][row] = Felt::ONE - trace[3][row - 1];
            trace[4][row] = Felt::ONE - trace[4][row - 1];
            trace[5][row] = trace[4][row - 1];
            trace[6][row] = trace[4][row - 1];
        }

        trace
    }
}

impl CrossBackendTestConfig for BusesComplexTestConfig {
    type WinterfellAir = WinterfellBusesComplexAir;
    type Plonky3Air = Plonky3BusesComplexAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        7
    }

    fn trace_length(&self) -> usize {
        self.trace_length
    }

    fn build_winterfell_trace(&self) -> Vec<Vec<Felt>> {
        self.build_trace()
    }

    fn build_winterfell_public_inputs(&self) -> PublicInputs {
        PublicInputs::new([Felt::ZERO; 2])
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellBusesComplexAir {
        WinterfellBusesComplexAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3BusesComplexAir {
        Plonky3BusesComplexAir
    }
}

#[test]
fn test_buses_complex_air_constraint_comparison() {
    let config = BusesComplexTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "BusesComplex AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_buses_complex_air_constraint_comparison_random_inputs() {
    let config = BusesComplexTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_buses_complex_air_constraint_comparison_random_inputs",
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

    println!("BusesComplex AIR random comparison passed for all 51 iterations (0-50)");
}
