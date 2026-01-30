//! Cross-backend comparison test for the BusesVarlenBoundaryLast AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the BusesVarlenBoundaryLast AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::buses::{
        buses_varlen_boundary_last::{
            BusesAir as WinterfellBusesVarlenBoundaryLastAir, PublicInputs,
        },
        buses_varlen_boundary_last_plonky3::BusesAir as Plonky3BusesVarlenBoundaryLastAir,
    },
};

/// Configuration for BusesVarlenBoundaryLast AIR cross-backend comparison tests.
struct BusesVarlenBoundaryLastTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl BusesVarlenBoundaryLastTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut trace = vec![vec![Felt::ZERO; length]; 5];

        trace[0][0] = Felt::ONE;
        for row in 1..length {
            trace[0][row] = Felt::ONE;
            trace[1][row] = if row > 3 && row < 8 { Felt::ONE } else { Felt::ZERO };
            trace[2][row] = if row > 3 && row < 7 { Felt::ONE } else { Felt::ZERO };
            trace[3][row] = if row > 4 && row < 10 { Felt::ONE } else { Felt::ZERO };
            trace[4][row] = if row > 5 && row < 10 { Felt::new(2) } else { Felt::ZERO };
        }

        trace
    }
}

impl CrossBackendTestConfig for BusesVarlenBoundaryLastTestConfig {
    type WinterfellAir = WinterfellBusesVarlenBoundaryLastAir;
    type Plonky3Air = Plonky3BusesVarlenBoundaryLastAir;
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
        PublicInputs::new(vec![[Felt::new(2), Felt::ZERO]], vec![[Felt::new(2), Felt::ZERO]])
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellBusesVarlenBoundaryLastAir {
        WinterfellBusesVarlenBoundaryLastAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3BusesVarlenBoundaryLastAir {
        Plonky3BusesVarlenBoundaryLastAir
    }
}

#[test]
#[ignore = "TODO: Winterfell codegen uses empty main degrees/assertions; fix goldens or codegen to satisfy AirContext::new_multi_segment"]
fn test_buses_varlen_boundary_last_air_constraint_comparison() {
    let config = BusesVarlenBoundaryLastTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "BusesVarlenBoundaryLast AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
#[ignore = "TODO: Winterfell codegen uses empty main degrees/assertions; fix goldens or codegen to satisfy AirContext::new_multi_segment"]
fn test_buses_varlen_boundary_last_air_constraint_comparison_random_inputs() {
    let config = BusesVarlenBoundaryLastTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_buses_varlen_boundary_last_air_constraint_comparison_random_inputs",
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

    println!("BusesVarlenBoundaryLast AIR random comparison passed for all 51 iterations (0-50)");
}
