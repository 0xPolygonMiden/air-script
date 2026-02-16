//! Expected OOD evaluations for the current tagged group.
//!
//! These values are captured from the miden-vm Rust constraints with seed 0xC0FFEE.

use air_mir::ir::QuadFelt;
use miden_core::Felt;

use super::manifest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvalRecord {
    pub id: usize,
    pub name: &'static str,
    pub value: QuadFelt,
}

pub const TOTAL_TAGS: usize = manifest::TOTAL_TAGS;

/// Stable constraint order as emitted by the miden-vm tagging pipeline.
/// This order must match the constraint IDs (0..16).
pub const CONSTRAINT_NAMES: [&str; TOTAL_TAGS] = [
    "system.clk.first_row",
    "system.clk.transition",
    "system.ctx.call_dyncall",
    "system.ctx.syscall",
    "system.ctx.default",
    "system.fn_hash.load",
    "system.fn_hash.load",
    "system.fn_hash.load",
    "system.fn_hash.load",
    "system.fn_hash.preserve",
    "system.fn_hash.preserve",
    "system.fn_hash.preserve",
    "system.fn_hash.preserve",
    "range.main.v.first_row",
    "range.main.v.last_row",
    "range.main.v.transition",
    "range.bus.transition",
];

/// Expected OOD values.
pub fn expected_ood_evals() -> Vec<EvalRecord> {
    vec![
        EvalRecord {
            id: 0,
            name: "system.clk.first_row",
            value: QuadFelt::new(Felt::new(1065013626484053923), Felt::new(0)),
        },
        EvalRecord {
            id: 1,
            name: "system.clk.transition",
            value: QuadFelt::new(Felt::new(5561241394822338942), Felt::new(0)),
        },
        EvalRecord {
            id: 2,
            name: "system.ctx.call_dyncall",
            value: QuadFelt::new(Felt::new(8631524473419082362), Felt::new(0)),
        },
        EvalRecord {
            id: 3,
            name: "system.ctx.syscall",
            value: QuadFelt::new(Felt::new(3242942367983627164), Felt::new(0)),
        },
        EvalRecord {
            id: 4,
            name: "system.ctx.default",
            value: QuadFelt::new(Felt::new(2699910395066589652), Felt::new(0)),
        },
        EvalRecord {
            id: 5,
            name: "system.fn_hash.load",
            value: QuadFelt::new(Felt::new(5171717963692258605), Felt::new(0)),
        },
        EvalRecord {
            id: 6,
            name: "system.fn_hash.load",
            value: QuadFelt::new(Felt::new(8961147296413400172), Felt::new(0)),
        },
        EvalRecord {
            id: 7,
            name: "system.fn_hash.load",
            value: QuadFelt::new(Felt::new(11894020196642675053), Felt::new(0)),
        },
        EvalRecord {
            id: 8,
            name: "system.fn_hash.load",
            value: QuadFelt::new(Felt::new(16889079421217525114), Felt::new(0)),
        },
        EvalRecord {
            id: 9,
            name: "system.fn_hash.preserve",
            value: QuadFelt::new(Felt::new(11909329801663906014), Felt::new(0)),
        },
        EvalRecord {
            id: 10,
            name: "system.fn_hash.preserve",
            value: QuadFelt::new(Felt::new(6717961555159342431), Felt::new(0)),
        },
        EvalRecord {
            id: 11,
            name: "system.fn_hash.preserve",
            value: QuadFelt::new(Felt::new(3950851291570048124), Felt::new(0)),
        },
        EvalRecord {
            id: 12,
            name: "system.fn_hash.preserve",
            value: QuadFelt::new(Felt::new(11146653144264413142), Felt::new(0)),
        },
        EvalRecord {
            id: 13,
            name: "range.main.v.first_row",
            value: QuadFelt::new(Felt::new(1112338059331632069), Felt::new(0)),
        },
        EvalRecord {
            id: 14,
            name: "range.main.v.last_row",
            value: QuadFelt::new(Felt::new(13352757668188868927), Felt::new(0)),
        },
        EvalRecord {
            id: 15,
            name: "range.main.v.transition",
            value: QuadFelt::new(Felt::new(12797082443503681195), Felt::new(0)),
        },
        EvalRecord {
            id: 16,
            name: "range.bus.transition",
            value: QuadFelt::new(Felt::new(10365289165200035540), Felt::new(16469718665506609592)),
        },
    ]
}
