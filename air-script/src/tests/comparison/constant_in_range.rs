//! Cross-backend comparison test for the ConstantInRange AIR.
//!
//! This test verifies that Winterfell and Plonky3 produce equivalent
//! constraint evaluations for the ConstantInRange AIR at every row of the trace.
//!
//! The ConstantInRange AIR tests comprehension over constant ranges:
//! - Boundary: `c[2].first = 0`
//! - Transition: `a = sum_{i=0..2}(i + b[i] - c[i] - d[i])`
//!
//! Trace columns: [a, b[3], c[4], d[4]] (12 total)
//! Column indices: a=0, b=1-3, c=4-7, d=8-11

use winter_air::{Air, ProofOptions as WinterProofOptions, TraceInfo};
use winter_math::{FieldElement, fields::f64::BaseElement as Felt};

use crate::{
    test_utils::cross_backend_comparison::{CrossBackendTestConfig, TraceSource, run_comparison},
    tests::constant_in_range::{
        constant_in_range::{ConstantInRangeAir as WinterfellConstantInRangeAir, PublicInputs},
        constant_in_range_plonky3::ConstantInRangeAir as Plonky3ConstantInRangeAir,
    },
};

/// Configuration for ConstantInRange AIR cross-backend comparison tests.
struct ConstantInRangeTestConfig {
    /// The trace length.
    trace_length: usize,
}

impl ConstantInRangeTestConfig {
    fn new(trace_length: usize) -> Self {
        Self { trace_length }
    }

    fn build_trace(&self) -> Vec<Vec<Felt>> {
        let length = self.trace_length;
        let mut trace = vec![vec![Felt::ZERO; length]; 12];

        for row in 0..length {
            let mut b = [Felt::ZERO; 3];
            let mut c = [Felt::ZERO; 3];
            let mut d = [Felt::ZERO; 3];

            for i in 0..3 {
                b[i] = Felt::new((row as u64) + (i as u64) + 1);
                c[i] = if row == 0 && i == 2 {
                    Felt::ZERO
                } else {
                    Felt::new((row as u64) + (i as u64) + 10)
                };
                d[i] = Felt::new((row as u64) + (i as u64) + 20);
            }

            let a =
                (0..3).fold(Felt::ZERO, |acc, i| acc + Felt::new(i as u64) + b[i] - c[i] - d[i]);

            trace[0][row] = a;
            trace[1][row] = b[0];
            trace[2][row] = b[1];
            trace[3][row] = b[2];
            trace[4][row] = c[0];
            trace[5][row] = c[1];
            trace[6][row] = c[2];
            trace[7][row] = Felt::new((row as u64) + 30);
            trace[8][row] = d[0];
            trace[9][row] = d[1];
            trace[10][row] = d[2];
            trace[11][row] = Felt::new((row as u64) + 40);
        }

        trace
    }
}

impl CrossBackendTestConfig for ConstantInRangeTestConfig {
    type WinterfellAir = WinterfellConstantInRangeAir;
    type Plonky3Air = Plonky3ConstantInRangeAir;
    type WinterfellPublicInputs = PublicInputs;

    fn trace_width(&self) -> usize {
        12
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
    ) -> WinterfellConstantInRangeAir {
        WinterfellConstantInRangeAir::new(trace_info, pub_inputs, options)
    }

    fn create_plonky3_air(&self) -> Plonky3ConstantInRangeAir {
        Plonky3ConstantInRangeAir
    }
}

#[test]
fn test_constant_in_range_air_constraint_comparison() {
    let config = ConstantInRangeTestConfig::new(64);
    let result = run_comparison(&config, TraceSource::Default);

    if !result.is_ok() {
        panic!("Constraint evaluation comparison failed!\n\n{}", result.format_report());
    }

    println!(
        "ConstantInRange AIR comparison passed: {} constraints checked across {} rows",
        result.total_constraints_checked, result.total_rows
    );
}

#[test]
fn test_constant_in_range_air_constraint_comparison_random_inputs() {
    let config = ConstantInRangeTestConfig::new(64);

    for iteration in 0u64..=50 {
        let result = run_comparison(
            &config,
            TraceSource::Random {
                test_name: "test_constant_in_range_air_constraint_comparison_random_inputs",
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

    println!("ConstantInRange AIR random comparison passed for all 51 iterations (0-50)");
}
