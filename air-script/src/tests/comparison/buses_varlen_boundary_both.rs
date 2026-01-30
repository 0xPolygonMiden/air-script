//! Cross-backend comparison test for the BusesVarlenBoundaryBoth AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the BusesVarlenBoundaryBoth AIR at every row of the trace.

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::buses::{
        buses_varlen_boundary_both::{
            BusesAir as WinterfellBusesVarlenBoundaryBothAir, PublicInputs,
        },
        buses_varlen_boundary_both_plonky3::BusesAir as Plonky3BusesVarlenBoundaryBothAir,
    },
};

/// Configuration for BusesVarlenBoundaryBoth AIR cross-backend comparison tests.
struct BusesVarlenBoundaryBothTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl BusesVarlenBoundaryBothTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut col0 = vec![Felt::ZERO; length];

        for row in 0..length {
            col0[row] = if row % 2 == 0 { Felt::ZERO } else { Felt::ONE };
        }

        vec![col0]
    }
}

impl CrossBackendTestConfig for BusesVarlenBoundaryBothTestConfig {
    type WinterfellAir = WinterfellBusesVarlenBoundaryBothAir;
    type Plonky3Air = Plonky3BusesVarlenBoundaryBothAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        1
    }

    fn trace_length(&self) -> usize {
        self.trace_length
    }

    fn build_winterfell_trace(&self) -> Vec<Vec<Felt>> {
        self.build_trace()
    }

    fn build_winterfell_public_inputs(&self) -> PublicInputs {
        PublicInputs::new(
            vec![[Felt::new(4), Felt::ZERO, Felt::ZERO, Felt::ZERO]],
            vec![[Felt::new(2), Felt::ZERO]],
        )
    }

    fn create_winterfell_air(
        &self,
        trace_info: TraceInfo,
        pub_inputs: PublicInputs,
        options: WinterProofOptions,
    ) -> WinterfellBusesVarlenBoundaryBothAir {
        WinterfellBusesVarlenBoundaryBothAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3BusesVarlenBoundaryBothAir {
        Plonky3BusesVarlenBoundaryBothAir
    }
}

#[test]
#[ignore = "TODO: Winterfell codegen uses empty main degrees/assertions; fix goldens or codegen to satisfy AirContext::new_multi_segment"]
fn test_buses_varlen_boundary_both_air_constraint_comparison() {
    let config = BusesVarlenBoundaryBothTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "BusesVarlenBoundaryBoth AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
#[ignore = "TODO: Winterfell codegen uses empty main degrees/assertions; fix goldens or codegen to satisfy AirContext::new_multi_segment"]
fn test_buses_varlen_boundary_both_air_constraint_comparison_random_inputs() {
    let config = BusesVarlenBoundaryBothTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_buses_varlen_boundary_both_air_constraint_comparison_random_inputs",
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

    println!("BusesVarlenBoundaryBoth AIR random comparison passed for all 51 iterations (0-50)");
}
