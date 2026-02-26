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
/// This order must match the constraint IDs (0..CURRENT_MAX_ID).
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
    "stack.general.transition.0",
    "stack.general.transition.1",
    "stack.general.transition.2",
    "stack.general.transition.3",
    "stack.general.transition.4",
    "stack.general.transition.5",
    "stack.general.transition.6",
    "stack.general.transition.7",
    "stack.general.transition.8",
    "stack.general.transition.9",
    "stack.general.transition.10",
    "stack.general.transition.11",
    "stack.general.transition.12",
    "stack.general.transition.13",
    "stack.general.transition.14",
    "stack.general.transition.15",
    "range.bus.transition",
    "stack.overflow.depth.first_row",
    "stack.overflow.depth.last_row",
    "stack.overflow.addr.first_row",
    "stack.overflow.addr.last_row",
    "stack.overflow.depth.transition",
    "stack.overflow.flag.transition",
    "stack.overflow.addr.transition",
    "stack.overflow.zero_insert.transition",
    "stack.overflow.bus.transition",
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
            name: "stack.general.transition.0",
            value: QuadFelt::new(Felt::new(2617308096902219240), Felt::new(0)),
        },
        EvalRecord {
            id: 17,
            name: "stack.general.transition.1",
            value: QuadFelt::new(Felt::new(4439102810547612775), Felt::new(0)),
        },
        EvalRecord {
            id: 18,
            name: "stack.general.transition.2",
            value: QuadFelt::new(Felt::new(15221140463513662734), Felt::new(0)),
        },
        EvalRecord {
            id: 19,
            name: "stack.general.transition.3",
            value: QuadFelt::new(Felt::new(4910128267170087966), Felt::new(0)),
        },
        EvalRecord {
            id: 20,
            name: "stack.general.transition.4",
            value: QuadFelt::new(Felt::new(8221884229886405628), Felt::new(0)),
        },
        EvalRecord {
            id: 21,
            name: "stack.general.transition.5",
            value: QuadFelt::new(Felt::new(87491100192562680), Felt::new(0)),
        },
        EvalRecord {
            id: 22,
            name: "stack.general.transition.6",
            value: QuadFelt::new(Felt::new(11411892308848385202), Felt::new(0)),
        },
        EvalRecord {
            id: 23,
            name: "stack.general.transition.7",
            value: QuadFelt::new(Felt::new(2425094460891103256), Felt::new(0)),
        },
        EvalRecord {
            id: 24,
            name: "stack.general.transition.8",
            value: QuadFelt::new(Felt::new(2767534397043537043), Felt::new(0)),
        },
        EvalRecord {
            id: 25,
            name: "stack.general.transition.9",
            value: QuadFelt::new(Felt::new(11686523590994044007), Felt::new(0)),
        },
        EvalRecord {
            id: 26,
            name: "stack.general.transition.10",
            value: QuadFelt::new(Felt::new(15000969044032170777), Felt::new(0)),
        },
        EvalRecord {
            id: 27,
            name: "stack.general.transition.11",
            value: QuadFelt::new(Felt::new(17422355615541008592), Felt::new(0)),
        },
        EvalRecord {
            id: 28,
            name: "stack.general.transition.12",
            value: QuadFelt::new(Felt::new(2555448945580115158), Felt::new(0)),
        },
        EvalRecord {
            id: 29,
            name: "stack.general.transition.13",
            value: QuadFelt::new(Felt::new(8864896307613509), Felt::new(0)),
        },
        EvalRecord {
            id: 30,
            name: "stack.general.transition.14",
            value: QuadFelt::new(Felt::new(3997062422665481459), Felt::new(0)),
        },
        EvalRecord {
            id: 31,
            name: "stack.general.transition.15",
            value: QuadFelt::new(Felt::new(6149720027324442163), Felt::new(0)),
        },
        EvalRecord {
            id: 32,
            name: "range.bus.transition",
            value: QuadFelt::new(Felt::new(10365289165200035540), Felt::new(16469718665506609592)),
        },
        EvalRecord {
            id: 33,
            name: "stack.overflow.depth.first_row",
            value: QuadFelt::new(Felt::new(1820735510664294085), Felt::new(0)),
        },
        EvalRecord {
            id: 34,
            name: "stack.overflow.depth.last_row",
            value: QuadFelt::new(Felt::new(12520055704510454391), Felt::new(0)),
        },
        EvalRecord {
            id: 35,
            name: "stack.overflow.addr.first_row",
            value: QuadFelt::new(Felt::new(9235172344178625178), Felt::new(0)),
        },
        EvalRecord {
            id: 36,
            name: "stack.overflow.addr.last_row",
            value: QuadFelt::new(Felt::new(6001883085148683205), Felt::new(0)),
        },
        EvalRecord {
            id: 37,
            name: "stack.overflow.depth.transition",
            value: QuadFelt::new(Felt::new(6706883717633639596), Felt::new(0)),
        },
        EvalRecord {
            id: 38,
            name: "stack.overflow.flag.transition",
            value: QuadFelt::new(Felt::new(5309566436521762910), Felt::new(0)),
        },
        EvalRecord {
            id: 39,
            name: "stack.overflow.addr.transition",
            value: QuadFelt::new(Felt::new(13739720401332236216), Felt::new(0)),
        },
        EvalRecord {
            id: 40,
            name: "stack.overflow.zero_insert.transition",
            value: QuadFelt::new(Felt::new(15830245309845547857), Felt::new(0)),
        },
        EvalRecord {
            id: 41,
            name: "stack.overflow.bus.transition",
            value: QuadFelt::new(Felt::new(7384164985445418427), Felt::new(3858806565449404456)),
        },
    ]
}
