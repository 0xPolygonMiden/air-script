//! OOD evaluation parity test for the minimal MidenVM AIR.
//!
//! We mirror the miden-vm evaluation flow and compare against fixed outputs captured from miden-vm.

use std::{collections::BTreeMap, sync::Arc};

use air_ir::{
    passes::{BusOpExpand, CommonSubexpressionElimination, MirToAir, TagValidation},
    Air, AlgebraicGraph, ConstraintDomain, ConstraintRoot, NodeIndex, Operation, TraceSegmentId,
    Value,
};
use air_mir::ir::QuadFelt;
use air_mir::MirPasses;
use air_parser::ast::BusConstraintForm;
use air_parser::AstPasses;
use air_pass::Pass;
use miden_core::{Felt, FieldElement};
use miden_diagnostics::{
    term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use winter_utils::Randomizable;

// === Constants ===

/// Seed used to generate deterministic OOD values (matches miden-vm tests).
const OOD_SEED: u64 = 0xC0FFEE;
/// Number of random extension elements expected by the OOD evaluator.
const AUX_TRACE_RAND_ELEMENTS: usize = 16;

/// Stable constraint order as emitted by the miden-vm tagging pipeline.
/// This order must match the constraint IDs (0..417).
const CONSTRAINT_NAMES: [&'static str; 418] = [
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
    "decoder.op_bits.b0.binary",
    "decoder.op_bits.b1.binary",
    "decoder.op_bits.b2.binary",
    "decoder.op_bits.b3.binary",
    "decoder.op_bits.b4.binary",
    "decoder.op_bits.b5.binary",
    "decoder.op_bits.b6.binary",
    "decoder.extra.e0",
    "decoder.extra.e1",
    "decoder.op_bits.u32_prefix.b0",
    "decoder.op_bits.very_high.b0",
    "decoder.op_bits.very_high.b1",
    "decoder.batch_flags.c0.binary",
    "decoder.batch_flags.c1.binary",
    "decoder.batch_flags.c2.binary",
    "decoder.general.split_loop.s0.binary",
    "decoder.general.dyn.h4.zero",
    "decoder.general.dyn.h5.zero",
    "decoder.general.dyn.h6.zero",
    "decoder.general.dyn.h7.zero",
    "decoder.general.repeat.s0.one",
    "decoder.general.repeat.h4.one",
    "decoder.general.end.loop.s0.zero",
    "decoder.general.end_repeat.h0.carry",
    "decoder.general.end_repeat.h1.carry",
    "decoder.general.end_repeat.h2.carry",
    "decoder.general.end_repeat.h3.carry",
    "decoder.general.end_repeat.h4.carry",
    "decoder.general.halt.next",
    "decoder.in_span.first_row",
    "decoder.in_span.binary",
    "decoder.in_span.span",
    "decoder.in_span.respan",
    "decoder.group_count.delta.binary",
    "decoder.group_count.decrement.h0_or_imm",
    "decoder.group_count.span_decrement",
    "decoder.group_count.end_or_respan.hold",
    "decoder.group_count.end.zero",
    "decoder.op_group.shift",
    "decoder.op_group.end_or_respan.h0.zero",
    "decoder.op_index.span_respan.reset",
    "decoder.op_index.new_group.reset",
    "decoder.op_index.increment",
    "decoder.op_index.range",
    "decoder.batch_flags.span_sum",
    "decoder.batch_flags.zero_when_not_span",
    "decoder.batch_flags.h4.zero",
    "decoder.batch_flags.h5.zero",
    "decoder.batch_flags.h6.zero",
    "decoder.batch_flags.h7.zero",
    "decoder.batch_flags.h2.zero",
    "decoder.batch_flags.h3.zero",
    "decoder.batch_flags.h1.zero",
    "decoder.addr.hold_in_span",
    "decoder.addr.respan.increment",
    "decoder.addr.halt.zero",
    "decoder.control_flow.sp_complement",
    "decoder.bus.p1.transition",
    "decoder.bus.p2.transition",
    "decoder.bus.p3.transition",
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
    "stack.overflow.depth.first_row",
    "stack.overflow.depth.last_row",
    "stack.overflow.addr.first_row",
    "stack.overflow.addr.last_row",
    "stack.overflow.depth.transition",
    "stack.overflow.flag.transition",
    "stack.overflow.addr.transition",
    "stack.overflow.zero_insert.transition",
    "stack.overflow.bus.transition",
    "stack.ops.pad",
    "stack.ops.dup",
    "stack.ops.dup",
    "stack.ops.dup",
    "stack.ops.dup",
    "stack.ops.dup",
    "stack.ops.dup",
    "stack.ops.dup",
    "stack.ops.dup",
    "stack.ops.dup",
    "stack.ops.dup",
    "stack.ops.dup",
    "stack.ops.dup",
    "stack.ops.clk",
    "stack.ops.swap",
    "stack.ops.swap",
    "stack.ops.movup",
    "stack.ops.movup",
    "stack.ops.movup",
    "stack.ops.movup",
    "stack.ops.movup",
    "stack.ops.movup",
    "stack.ops.movup",
    "stack.ops.movdn",
    "stack.ops.movdn",
    "stack.ops.movdn",
    "stack.ops.movdn",
    "stack.ops.movdn",
    "stack.ops.movdn",
    "stack.ops.movdn",
    "stack.ops.swapw",
    "stack.ops.swapw",
    "stack.ops.swapw",
    "stack.ops.swapw",
    "stack.ops.swapw",
    "stack.ops.swapw",
    "stack.ops.swapw",
    "stack.ops.swapw",
    "stack.ops.swapw2",
    "stack.ops.swapw2",
    "stack.ops.swapw2",
    "stack.ops.swapw2",
    "stack.ops.swapw2",
    "stack.ops.swapw2",
    "stack.ops.swapw2",
    "stack.ops.swapw2",
    "stack.ops.swapw3",
    "stack.ops.swapw3",
    "stack.ops.swapw3",
    "stack.ops.swapw3",
    "stack.ops.swapw3",
    "stack.ops.swapw3",
    "stack.ops.swapw3",
    "stack.ops.swapw3",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.swapdw",
    "stack.ops.cswap",
    "stack.ops.cswap",
    "stack.ops.cswap",
    "stack.ops.cswapw",
    "stack.ops.cswapw",
    "stack.ops.cswapw",
    "stack.ops.cswapw",
    "stack.ops.cswapw",
    "stack.ops.cswapw",
    "stack.ops.cswapw",
    "stack.ops.cswapw",
    "stack.ops.cswapw",
    "stack.system.assert",
    "stack.system.caller",
    "stack.system.caller",
    "stack.system.caller",
    "stack.system.caller",
    "stack.io.sdepth",
    "stack.field.add.transition",
    "stack.field.neg.transition",
    "stack.field.mul.transition",
    "stack.field.inv.transition",
    "stack.field.incr.transition",
    "stack.field.not.binary",
    "stack.field.not.result",
    "stack.field.and.binary.a",
    "stack.field.and.binary.b",
    "stack.field.and.result",
    "stack.field.or.binary.a",
    "stack.field.or.binary.b",
    "stack.field.or.result",
    "stack.field.eq.zero_product",
    "stack.field.eq.result",
    "stack.field.eqz.zero_product",
    "stack.field.eqz.result",
    "stack.field.expacc.square",
    "stack.field.expacc.val",
    "stack.field.expacc.acc",
    "stack.field.expacc.shift",
    "stack.field.expacc.bit_binary",
    "stack.field.ext2mul.d0",
    "stack.field.ext2mul.d1",
    "stack.field.ext2mul.c0",
    "stack.field.ext2mul.c1",
    "stack.u32.validity",
    "stack.u32.limbs.lo",
    "stack.u32.limbs.hi",
    "stack.u32.split",
    "stack.u32.add",
    "stack.u32.add3",
    "stack.u32.sub.agg",
    "stack.u32.sub.borrow",
    "stack.u32.sub.diff",
    "stack.u32.mul",
    "stack.u32.madd",
    "stack.u32.div.agg",
    "stack.u32.div.quot_bound",
    "stack.u32.div.rem_bound",
    "stack.u32.assert2.b",
    "stack.u32.assert2.a",
    "stack.crypto.cryptostream.capacity.0",
    "stack.crypto.cryptostream.capacity.1",
    "stack.crypto.cryptostream.capacity.2",
    "stack.crypto.cryptostream.capacity.3",
    "stack.crypto.cryptostream.ptr.src",
    "stack.crypto.cryptostream.ptr.dst",
    "stack.crypto.cryptostream.tail.0",
    "stack.crypto.cryptostream.tail.1",
    "stack.crypto.hornerbase.stack.0",
    "stack.crypto.hornerbase.stack.1",
    "stack.crypto.hornerbase.stack.2",
    "stack.crypto.hornerbase.stack.3",
    "stack.crypto.hornerbase.stack.4",
    "stack.crypto.hornerbase.stack.5",
    "stack.crypto.hornerbase.stack.6",
    "stack.crypto.hornerbase.stack.7",
    "stack.crypto.hornerbase.stack.8",
    "stack.crypto.hornerbase.stack.9",
    "stack.crypto.hornerbase.stack.10",
    "stack.crypto.hornerbase.stack.11",
    "stack.crypto.hornerbase.stack.12",
    "stack.crypto.hornerbase.stack.13",
    "stack.crypto.hornerbase.tmp0.0",
    "stack.crypto.hornerbase.tmp0.1",
    "stack.crypto.hornerbase.tmp1.0",
    "stack.crypto.hornerbase.tmp1.1",
    "stack.crypto.hornerbase.acc.0",
    "stack.crypto.hornerbase.acc.1",
    "stack.crypto.hornerext.stack.0",
    "stack.crypto.hornerext.stack.1",
    "stack.crypto.hornerext.stack.2",
    "stack.crypto.hornerext.stack.3",
    "stack.crypto.hornerext.stack.4",
    "stack.crypto.hornerext.stack.5",
    "stack.crypto.hornerext.stack.6",
    "stack.crypto.hornerext.stack.7",
    "stack.crypto.hornerext.stack.8",
    "stack.crypto.hornerext.stack.9",
    "stack.crypto.hornerext.stack.10",
    "stack.crypto.hornerext.stack.11",
    "stack.crypto.hornerext.stack.12",
    "stack.crypto.hornerext.stack.13",
    "stack.crypto.hornerext.tmp.0",
    "stack.crypto.hornerext.tmp.1",
    "stack.crypto.hornerext.acc.0",
    "stack.crypto.hornerext.acc.1",
    "range.main.v.first_row",
    "range.main.v.last_row",
    "range.main.v.transition",
    "range.bus.transition",
    "chiplets.selectors.s0.binary",
    "chiplets.selectors.s1.binary",
    "chiplets.selectors.s2.binary",
    "chiplets.selectors.s3.binary",
    "chiplets.selectors.s4.binary",
    "chiplets.selectors.s0.stability",
    "chiplets.selectors.s1.stability",
    "chiplets.selectors.s2.stability",
    "chiplets.selectors.s3.stability",
    "chiplets.selectors.s4.stability",
    "chiplets.hasher.permutation.init",
    "chiplets.hasher.permutation.init",
    "chiplets.hasher.permutation.init",
    "chiplets.hasher.permutation.init",
    "chiplets.hasher.permutation.init",
    "chiplets.hasher.permutation.init",
    "chiplets.hasher.permutation.init",
    "chiplets.hasher.permutation.init",
    "chiplets.hasher.permutation.init",
    "chiplets.hasher.permutation.init",
    "chiplets.hasher.permutation.init",
    "chiplets.hasher.permutation.init",
    "chiplets.hasher.permutation.external",
    "chiplets.hasher.permutation.external",
    "chiplets.hasher.permutation.external",
    "chiplets.hasher.permutation.external",
    "chiplets.hasher.permutation.external",
    "chiplets.hasher.permutation.external",
    "chiplets.hasher.permutation.external",
    "chiplets.hasher.permutation.external",
    "chiplets.hasher.permutation.external",
    "chiplets.hasher.permutation.external",
    "chiplets.hasher.permutation.external",
    "chiplets.hasher.permutation.external",
    "chiplets.hasher.permutation.internal",
    "chiplets.hasher.permutation.internal",
    "chiplets.hasher.permutation.internal",
    "chiplets.hasher.permutation.internal",
    "chiplets.hasher.permutation.internal",
    "chiplets.hasher.permutation.internal",
    "chiplets.hasher.permutation.internal",
    "chiplets.hasher.permutation.internal",
    "chiplets.hasher.permutation.internal",
    "chiplets.hasher.permutation.internal",
    "chiplets.hasher.permutation.internal",
    "chiplets.hasher.permutation.internal",
    "chiplets.hasher.selectors.binary",
    "chiplets.hasher.selectors.binary",
    "chiplets.hasher.selectors.binary",
    "chiplets.hasher.selectors.stability",
    "chiplets.hasher.selectors.stability",
    "chiplets.hasher.selectors.continuation",
    "chiplets.hasher.selectors.invalid",
    "chiplets.hasher.abp.capacity",
    "chiplets.hasher.abp.capacity",
    "chiplets.hasher.abp.capacity",
    "chiplets.hasher.abp.capacity",
    "chiplets.hasher.output.index",
    "chiplets.hasher.merkle.index.binary",
    "chiplets.hasher.merkle.index.stability",
    "chiplets.hasher.merkle.index.output",
    "chiplets.hasher.merkle.capacity",
    "chiplets.hasher.merkle.capacity",
    "chiplets.hasher.merkle.capacity",
    "chiplets.hasher.merkle.capacity",
    "chiplets.hasher.merkle.digest.rate0",
    "chiplets.hasher.merkle.digest.rate0",
    "chiplets.hasher.merkle.digest.rate0",
    "chiplets.hasher.merkle.digest.rate0",
    "chiplets.hasher.merkle.digest.rate1",
    "chiplets.hasher.merkle.digest.rate1",
    "chiplets.hasher.merkle.digest.rate1",
    "chiplets.hasher.merkle.digest.rate1",
    "chiplets.bitwise.op.binary",
    "chiplets.bitwise.op.stability",
    "chiplets.bitwise.a_bits.binary",
    "chiplets.bitwise.a_bits.binary",
    "chiplets.bitwise.a_bits.binary",
    "chiplets.bitwise.a_bits.binary",
    "chiplets.bitwise.b_bits.binary",
    "chiplets.bitwise.b_bits.binary",
    "chiplets.bitwise.b_bits.binary",
    "chiplets.bitwise.b_bits.binary",
    "chiplets.bitwise.first_row",
    "chiplets.bitwise.first_row",
    "chiplets.bitwise.first_row",
    "chiplets.bitwise.input.transition",
    "chiplets.bitwise.input.transition",
    "chiplets.bitwise.output.prev",
    "chiplets.bitwise.output.aggregate",
    "chiplets.memory.binary",
    "chiplets.memory.binary",
    "chiplets.memory.binary",
    "chiplets.memory.binary",
    "chiplets.memory.word_idx.zero",
    "chiplets.memory.word_idx.zero",
    "chiplets.memory.first_row.zero",
    "chiplets.memory.first_row.zero",
    "chiplets.memory.first_row.zero",
    "chiplets.memory.first_row.zero",
    "chiplets.memory.delta.inv",
    "chiplets.memory.delta.inv",
    "chiplets.memory.delta.inv",
    "chiplets.memory.delta.inv",
    "chiplets.memory.delta.transition",
    "chiplets.memory.scw.flag",
    "chiplets.memory.scw.reads",
    "chiplets.memory.value.consistency",
    "chiplets.memory.value.consistency",
    "chiplets.memory.value.consistency",
    "chiplets.memory.value.consistency",
    "chiplets.ace.selector.binary",
    "chiplets.ace.selector.binary",
    "chiplets.ace.section.flags",
    "chiplets.ace.section.flags",
    "chiplets.ace.section.flags",
    "chiplets.ace.section.flags",
    "chiplets.ace.section.flags",
    "chiplets.ace.section.transition",
    "chiplets.ace.section.transition",
    "chiplets.ace.section.transition",
    "chiplets.ace.section.transition",
    "chiplets.ace.read.ids",
    "chiplets.ace.read.to_eval",
    "chiplets.ace.eval.op",
    "chiplets.ace.eval.result",
    "chiplets.ace.eval.result",
    "chiplets.ace.final.zero",
    "chiplets.ace.final.zero",
    "chiplets.ace.final.zero",
    "chiplets.ace.first_row.start",
    "chiplets.kernel_rom.sfirst.binary",
    "chiplets.kernel_rom.digest.contiguity",
    "chiplets.kernel_rom.digest.contiguity",
    "chiplets.kernel_rom.digest.contiguity",
    "chiplets.kernel_rom.digest.contiguity",
    "chiplets.kernel_rom.first_row.start",
    "chiplets.bus.hash_kernel.transition",
    "chiplets.bus.chiplets.transition",
    "chiplets.bus.wiring.transition",
];

/// Transition-domain constraints that should NOT be multiplied by the transition selector.
///
/// These are already gated by `k_transition` in the AIR expression and are *not* wrapped in
/// `when_transition()` on the Rust side. If we also multiply them by `t` here, we double‑gate
/// the constraint and the OOD value diverges from miden-vm.
const UNGATED_TRANSITION_TAGS: [usize; 4] = [352, 364, 365, 366];

/// Periodic columns are consumed in the same order as miden-vm (hasher periodic columns).
const PERIODIC_COLUMN_ORDER: [&str; 20] = [
    "cycle_row_0",
    "cycle_row_30",
    "cycle_row_31",
    "p2_is_external",
    "p2_is_internal",
    "ark_ext_0",
    "ark_ext_1",
    "ark_ext_2",
    "ark_ext_3",
    "ark_ext_4",
    "ark_ext_5",
    "ark_ext_6",
    "ark_ext_7",
    "ark_ext_8",
    "ark_ext_9",
    "ark_ext_10",
    "ark_ext_11",
    "ark_int",
    "k_first",
    "k_transition",
];

// === Helpers ===

/// Returns the path to the minimal MidenVM AIR example.
pub fn load_miden_vm_minimal_air() -> std::io::Result<String> {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = format!("{}/minimal/miden_vm_minimal.air", crate_dir);
    Ok(path)
}

/// Parses and lowers AIRScript into AIR for evaluation.
fn generate_air(path: &str) -> Air {
    if std::env::var("AIR_ENABLE_CSE").is_ok() {
        generate_air_with_cse(path)
    } else {
        generate_air_without_cse(path)
    }
}

/// Parses and lowers AIRScript into AIR for evaluation (no CSE).
fn generate_air_without_cse(path: &str) -> Air {
    let code_map = Arc::new(CodeMap::new());
    let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
    let diagnostics = DiagnosticsHandler::new(Default::default(), code_map.clone(), emitter);

    let program = air_parser::parse_file(&diagnostics, code_map, path)
        .map_err(air_ir::CompileError::Parse)
        .expect("parse failed");

    let ast_passes = AstPasses::new(&diagnostics);
    let mir_passes = MirPasses::new(&diagnostics);
    let air_passes = MirToAir::new(&diagnostics)
        .chain(BusOpExpand::new(&diagnostics))
        .chain(TagValidation::new(&diagnostics));

    let air = ast_passes
        .chain(mir_passes)
        .chain(air_passes)
        .run(program)
        .expect("lowering failed");

    air
}

/// Parses and lowers AIRScript into AIR for evaluation (with CSE enabled).
fn generate_air_with_cse(path: &str) -> Air {
    let code_map = Arc::new(CodeMap::new());
    let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
    let diagnostics = DiagnosticsHandler::new(Default::default(), code_map.clone(), emitter);

    let program = air_parser::parse_file(&diagnostics, code_map, path)
        .map_err(air_ir::CompileError::Parse)
        .expect("parse failed");

    let ast_passes = AstPasses::new(&diagnostics);
    let mir_passes = MirPasses::new(&diagnostics);
    let air_passes = MirToAir::new(&diagnostics)
        .chain(BusOpExpand::new(&diagnostics))
        .chain(CommonSubexpressionElimination::new(&diagnostics))
        .chain(TagValidation::new(&diagnostics));

    let air = ast_passes
        .chain(mir_passes)
        .chain(air_passes)
        .run(program)
        .expect("lowering failed");

    air
}

/// Deterministic PRNG value generator matching miden-vm's OOD helper.
fn prng_value<T: Randomizable>(seed: [u8; 32]) -> T {
    let mut rng = ChaCha20Rng::from_seed(seed);
    let mut bytes = vec![0u8; T::VALUE_SIZE];
    rng.fill(&mut bytes[..]);
    T::from_random_bytes(&bytes).expect("failed to generate random value")
}

/// Deterministic RNG with the same seed/counter mixing as miden-vm.
/// Keeps RNG consumption order aligned with the Rust implementation.
struct SeededRng {
    seed: u64,
    counter: u64,
}

impl SeededRng {
    fn new(seed: u64) -> Self {
        Self { seed, counter: 0 }
    }

    fn next_felt(&mut self) -> Felt {
        let bytes = self.next_seed_bytes();
        prng_value::<Felt>(bytes)
    }

    fn next_quad(&mut self) -> QuadFelt {
        QuadFelt::new(self.next_felt(), self.next_felt())
    }

    /// Derives the next 32-byte seed by mixing `seed` and `counter`.
    fn next_seed_bytes(&mut self) -> [u8; 32] {
        let counter = self.counter;
        self.counter = self.counter.wrapping_add(1);
        let mix = self.seed ^ counter;
        let sum = self.seed.wrapping_add(counter);
        let mut out = [0u8; 32];
        out[0..8].copy_from_slice(&self.seed.to_le_bytes());
        out[8..16].copy_from_slice(&counter.to_le_bytes());
        out[16..24].copy_from_slice(&mix.to_le_bytes());
        out[24..32].copy_from_slice(&sum.to_le_bytes());
        out
    }
}

// === Types ===

/// OOD eval uses Goldilocks quadratic extension (x^2 - 7) to match Plonky3/Miden VM.
/// This is a test-only workaround until we depend on Plonky3 types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct QuadGoldilocksOOD {
    c0: Felt,
    c1: Felt,
}

impl QuadGoldilocksOOD {
    const ZERO: Self = Self { c0: Felt::ZERO, c1: Felt::ZERO };
    const ONE: Self = Self { c0: Felt::ONE, c1: Felt::ZERO };
    fn from_felt(value: Felt) -> Self {
        Self { c0: value, c1: Felt::ZERO }
    }

    fn from_quad(value: QuadFelt) -> Self {
        let [c0, c1] = value.to_base_elements();
        Self { c0, c1 }
    }

    fn to_quad(self) -> QuadFelt {
        QuadFelt::new(self.c0, self.c1)
    }

    fn inv(self) -> Self {
        let non_residue = Felt::new(7);
        let a = self.c0;
        let b = self.c1;
        let denom = a * a - non_residue * (b * b);
        let denom_inv = denom.inv();
        Self {
            c0: a * denom_inv,
            c1: (Felt::ZERO - b) * denom_inv,
        }
    }
}

impl std::ops::Add for QuadGoldilocksOOD {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            c0: self.c0 + rhs.c0,
            c1: self.c1 + rhs.c1,
        }
    }
}

impl std::ops::Sub for QuadGoldilocksOOD {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            c0: self.c0 - rhs.c0,
            c1: self.c1 - rhs.c1,
        }
    }
}

impl std::ops::Mul for QuadGoldilocksOOD {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // Goldilocks quadratic extension: x^2 - 7.
        let a0 = self.c0;
        let a1 = self.c1;
        let b0 = rhs.c0;
        let b1 = rhs.c1;
        let non_residue = Felt::new(7);

        Self {
            c0: a0 * b0 + non_residue * (a1 * b1),
            c1: a0 * b1 + a1 * b0,
        }
    }
}

/// Captures the random OOD evaluation context (trace rows, challenges, row flags).
#[derive(Clone)]
struct OodContext {
    /// Two consecutive main-trace rows at the OOD point.
    main_rows: [Vec<Felt>; 2],
    /// Two consecutive aux-trace rows at the OOD point.
    aux_rows: [Vec<QuadGoldilocksOOD>; 2],
    /// Periodic column values keyed by column name.
    periodic_values: BTreeMap<&'static str, Felt>,
    /// Random values consumed by the AIR (alpha, 1, then powers of beta).
    random_values: Vec<QuadGoldilocksOOD>,
    /// First-row selector.
    first_row: QuadGoldilocksOOD,
    /// Last-row selector.
    last_row: QuadGoldilocksOOD,
    /// Transition selector.
    transition: QuadGoldilocksOOD,
}

impl OodContext {
    /// Builds an OOD context whose RNG consumption order matches miden-vm.
    fn new(air: &Air, seed: u64) -> Self {
        let main_width = air.trace_segment_widths[usize::from(TraceSegmentId::Main)] as usize;
        let aux_width = air.trace_segment_widths[usize::from(TraceSegmentId::Aux)] as usize;
        assert_eq!(main_width, 72, "unexpected main trace width for minimal AIR");
        assert_eq!(aux_width, 8, "unexpected aux trace width for minimal AIR");
        assert_eq!(
            usize::from(air.num_random_values),
            AUX_TRACE_RAND_ELEMENTS,
            "unexpected randomness count for minimal AIR"
        );

        let mut rng = SeededRng::new(seed);

        // miden-vm uses TRACE_WIDTH=71 (no padding). Keep padding at zero without consuming RNG.
        let main_filled = main_width - 1;
        let mut main_rows = [vec![Felt::new(0); main_width], vec![Felt::new(0); main_width]];
        for row in 0..2 {
            for col in 0..main_filled {
                main_rows[row][col] = rng.next_felt();
            }
        }

        let mut aux_rows = [
            vec![QuadGoldilocksOOD::ZERO; aux_width],
            vec![QuadGoldilocksOOD::ZERO; aux_width],
        ];
        for row in 0..2 {
            for col in 0..aux_width {
                aux_rows[row][col] = QuadGoldilocksOOD::from_quad(rng.next_quad());
            }
        }

        let mut randomness = Vec::with_capacity(AUX_TRACE_RAND_ELEMENTS);
        for _ in 0..AUX_TRACE_RAND_ELEMENTS {
            randomness.push(QuadGoldilocksOOD::from_quad(rng.next_quad()));
        }
        // Consume aux bus boundary values to match miden-vm RNG sequence.
        for _ in 0..aux_width {
            let _ = rng.next_quad();
        }

        let first_row = QuadGoldilocksOOD::from_felt(rng.next_felt());
        let last_row = QuadGoldilocksOOD::from_felt(rng.next_felt());
        let transition = QuadGoldilocksOOD::from_felt(rng.next_felt());
        assert_eq!(
            air.periodic_columns.len(),
            PERIODIC_COLUMN_ORDER.len(),
            "unexpected periodic column count for minimal AIR"
        );
        let mut periodic_values = BTreeMap::new();
        for name in PERIODIC_COLUMN_ORDER {
            periodic_values.insert(name, rng.next_felt());
        }

        let alpha = *randomness.first().expect("randomness missing alpha for bus challenges");
        let beta = *randomness.get(1).expect("randomness missing beta for bus challenges");

        let mut random_values = Vec::with_capacity(air.num_random_values as usize);
        if air.num_random_values > 0 {
            random_values.push(alpha);
        }
        if air.num_random_values > 1 {
            random_values.push(QuadGoldilocksOOD::from_felt(Felt::ONE));
        }
        if air.num_random_values > 2 {
            let mut beta_power = beta;
            for _ in 2..air.num_random_values as usize {
                random_values.push(beta_power);
                beta_power = beta_power * beta;
            }
        }

        Self {
            main_rows,
            aux_rows,
            periodic_values,
            random_values,
            first_row,
            last_row,
            transition,
        }
    }
}

// === OOD eval helpers ===

/// Evaluates constraints in tag order to match `CONSTRAINT_NAMES`.
/// The order is aligned with constraint IDs (boundary/integrity buckets).
fn eval_constraints_in_id_order(air: &Air, ctx: &OodContext) -> Vec<QuadGoldilocksOOD> {
    let mut tagged = Vec::new();
    for segment in [TraceSegmentId::Main, TraceSegmentId::Aux] {
        if usize::from(segment) >= air.trace_segment_widths.len() {
            continue;
        }
        for constraint in air.boundary_constraints(segment) {
            let tag = constraint.tag().expect("boundary constraint missing tag");
            tagged.push((tag, constraint));
        }
        for constraint in air.integrity_constraints(segment) {
            let tag = constraint.tag().expect("integrity constraint missing tag");
            tagged.push((tag, constraint));
        }
    }

    tagged.sort_by_key(|(tag, _)| *tag);
    assert_eq!(tagged.len(), CONSTRAINT_NAMES.len());
    for (expected, (tag, _)) in tagged.iter().enumerate() {
        assert_eq!(*tag as usize, expected, "unexpected tag ordering");
    }

    let graph = air.constraint_graph();
    let mut cache = vec![None; graph.num_nodes()];
    tagged
        .into_iter()
        .map(|(tag, constraint)| {
            let value = eval_constraint(graph, constraint, ctx, &mut cache);
            apply_domain_gate(value, constraint.domain(), tag as usize, ctx)
        })
        .collect()
}

/// Collects constraint tags with their segment/domain, sorted by tag.
fn collect_tagged_constraints(air: &Air) -> Vec<(u64, TraceSegmentId, ConstraintDomain)> {
    let mut tagged = Vec::new();
    for segment in [TraceSegmentId::Main, TraceSegmentId::Aux] {
        if usize::from(segment) >= air.trace_segment_widths.len() {
            continue;
        }
        for constraint in air.boundary_constraints(segment) {
            let tag = constraint.tag().expect("boundary constraint missing tag");
            tagged.push((tag, segment, constraint.domain()));
        }
        for constraint in air.integrity_constraints(segment) {
            let tag = constraint.tag().expect("integrity constraint missing tag");
            tagged.push((tag, segment, constraint.domain()));
        }
    }

    tagged.sort_by_key(|(tag, ..)| *tag);
    tagged
}

/// Evaluates a constraint root and applies its row-domain gate.
fn eval_constraint(
    graph: &AlgebraicGraph,
    root: &ConstraintRoot,
    ctx: &OodContext,
    cache: &mut [Option<QuadGoldilocksOOD>],
) -> QuadGoldilocksOOD {
    eval_node(graph, root.node_index(), ctx, cache)
}

fn apply_domain_gate(
    value: QuadGoldilocksOOD,
    domain: ConstraintDomain,
    tag: usize,
    ctx: &OodContext,
) -> QuadGoldilocksOOD {
    match domain {
        ConstraintDomain::FirstRow => value * ctx.first_row,
        ConstraintDomain::LastRow => value * ctx.last_row,
        ConstraintDomain::EveryFrame(2) => {
            if UNGATED_TRANSITION_TAGS.contains(&tag) {
                value
            } else {
                value * ctx.transition
            }
        },
        ConstraintDomain::EveryRow => value,
        ConstraintDomain::EveryFrame(size) => {
            panic!("unsupported transition size {size} for minimal OOD eval");
        },
    }
}

/// Evaluates a single algebraic graph node at the OOD point.
fn eval_node(
    graph: &AlgebraicGraph,
    index: &NodeIndex,
    ctx: &OodContext,
    cache: &mut [Option<QuadGoldilocksOOD>],
) -> QuadGoldilocksOOD {
    let mut stack = vec![*index];
    while let Some(node) = stack.pop() {
        let idx: usize = node.into();
        if cache[idx].is_some() {
            continue;
        }

        let value = match graph.node(&node).op() {
            Operation::Value(Value::Constant(value)) => {
                Some(QuadGoldilocksOOD::from_felt(Felt::new(*value)))
            },
            Operation::Value(Value::RandomValue(index)) => {
                Some(ctx.random_values.get(*index).copied().expect("random index out of bounds"))
            },
            Operation::Value(Value::TraceAccess(access)) => {
                let row = access.row_offset;
                let value = match access.segment {
                    TraceSegmentId::Main => {
                        let felt_value = ctx
                            .main_rows
                            .get(row)
                            .and_then(|row| row.get(access.column))
                            .copied()
                            .expect("main trace access out of bounds");
                        QuadGoldilocksOOD::from_felt(felt_value)
                    },
                    TraceSegmentId::Aux => ctx
                        .aux_rows
                        .get(row)
                        .and_then(|row| row.get(access.column))
                        .copied()
                        .expect("aux trace access out of bounds"),
                };
                Some(value)
            },
            Operation::Value(Value::PeriodicColumn(index)) => {
                let name = index.name.as_ref().as_str();
                let value =
                    ctx.periodic_values.get(name).copied().expect("periodic column name not found");
                Some(QuadGoldilocksOOD::from_felt(value))
            },
            Operation::Value(Value::PublicInput(_)) => {
                panic!("public inputs are not expected in minimal OOD eval");
            },
            Operation::Value(Value::PublicInputTable(_)) => {
                panic!("public input tables are not expected in minimal OOD eval");
            },
            Operation::Add(lhs, rhs) => {
                let lhs_idx: usize = (*lhs).into();
                let rhs_idx: usize = (*rhs).into();
                match (cache[lhs_idx], cache[rhs_idx]) {
                    (Some(lhs_val), Some(rhs_val)) => Some(lhs_val + rhs_val),
                    _ => {
                        stack.push(node);
                        if cache[lhs_idx].is_none() {
                            stack.push(*lhs);
                        }
                        if cache[rhs_idx].is_none() {
                            stack.push(*rhs);
                        }
                        None
                    },
                }
            },
            Operation::Sub(lhs, rhs) => {
                let lhs_idx: usize = (*lhs).into();
                let rhs_idx: usize = (*rhs).into();
                match (cache[lhs_idx], cache[rhs_idx]) {
                    (Some(lhs_val), Some(rhs_val)) => Some(lhs_val - rhs_val),
                    _ => {
                        stack.push(node);
                        if cache[lhs_idx].is_none() {
                            stack.push(*lhs);
                        }
                        if cache[rhs_idx].is_none() {
                            stack.push(*rhs);
                        }
                        None
                    },
                }
            },
            Operation::Mul(lhs, rhs) => {
                let lhs_idx: usize = (*lhs).into();
                let rhs_idx: usize = (*rhs).into();
                match (cache[lhs_idx], cache[rhs_idx]) {
                    (Some(lhs_val), Some(rhs_val)) => Some(lhs_val * rhs_val),
                    _ => {
                        stack.push(node);
                        if cache[lhs_idx].is_none() {
                            stack.push(*lhs);
                        }
                        if cache[rhs_idx].is_none() {
                            stack.push(*rhs);
                        }
                        None
                    },
                }
            },
        };

        if let Some(value) = value {
            cache[idx] = Some(value);
        }
    }

    let idx: usize = (*index).into();
    cache[idx].expect("node evaluation missing")
}

// === OOD expected outputs ===

fn expected_ood_evals() -> Vec<(&'static str, QuadFelt)> {
    vec![
        (
            "system.clk.first_row",
            QuadFelt::new(Felt::new(1065013626484053923), Felt::new(0)),
        ),
        (
            "system.clk.transition",
            QuadFelt::new(Felt::new(5561241394822338942), Felt::new(0)),
        ),
        (
            "system.ctx.call_dyncall",
            QuadFelt::new(Felt::new(8631524473419082362), Felt::new(0)),
        ),
        (
            "system.ctx.syscall",
            QuadFelt::new(Felt::new(3242942367983627164), Felt::new(0)),
        ),
        (
            "system.ctx.default",
            QuadFelt::new(Felt::new(2699910395066589652), Felt::new(0)),
        ),
        (
            "system.fn_hash.load",
            QuadFelt::new(Felt::new(5171717963692258605), Felt::new(0)),
        ),
        (
            "system.fn_hash.load",
            QuadFelt::new(Felt::new(8961147296413400172), Felt::new(0)),
        ),
        (
            "system.fn_hash.load",
            QuadFelt::new(Felt::new(11894020196642675053), Felt::new(0)),
        ),
        (
            "system.fn_hash.load",
            QuadFelt::new(Felt::new(16889079421217525114), Felt::new(0)),
        ),
        (
            "system.fn_hash.preserve",
            QuadFelt::new(Felt::new(11909329801663906014), Felt::new(0)),
        ),
        (
            "system.fn_hash.preserve",
            QuadFelt::new(Felt::new(6717961555159342431), Felt::new(0)),
        ),
        (
            "system.fn_hash.preserve",
            QuadFelt::new(Felt::new(3950851291570048124), Felt::new(0)),
        ),
        (
            "system.fn_hash.preserve",
            QuadFelt::new(Felt::new(11146653144264413142), Felt::new(0)),
        ),
        (
            "decoder.op_bits.b0.binary",
            QuadFelt::new(Felt::new(13628791071868321124), Felt::new(0)),
        ),
        (
            "decoder.op_bits.b1.binary",
            QuadFelt::new(Felt::new(2117480814916000258), Felt::new(0)),
        ),
        (
            "decoder.op_bits.b2.binary",
            QuadFelt::new(Felt::new(16926933246570374887), Felt::new(0)),
        ),
        (
            "decoder.op_bits.b3.binary",
            QuadFelt::new(Felt::new(9176310969543325496), Felt::new(0)),
        ),
        (
            "decoder.op_bits.b4.binary",
            QuadFelt::new(Felt::new(7537316481676351991), Felt::new(0)),
        ),
        (
            "decoder.op_bits.b5.binary",
            QuadFelt::new(Felt::new(2144456409708417452), Felt::new(0)),
        ),
        (
            "decoder.op_bits.b6.binary",
            QuadFelt::new(Felt::new(4533994350960751386), Felt::new(0)),
        ),
        ("decoder.extra.e0", QuadFelt::new(Felt::new(8133745730975361882), Felt::new(0))),
        ("decoder.extra.e1", QuadFelt::new(Felt::new(1382945310839592478), Felt::new(0))),
        (
            "decoder.op_bits.u32_prefix.b0",
            QuadFelt::new(Felt::new(3295186688501169293), Felt::new(0)),
        ),
        (
            "decoder.op_bits.very_high.b0",
            QuadFelt::new(Felt::new(1492924210658182178), Felt::new(0)),
        ),
        (
            "decoder.op_bits.very_high.b1",
            QuadFelt::new(Felt::new(11514104647859742926), Felt::new(0)),
        ),
        (
            "decoder.batch_flags.c0.binary",
            QuadFelt::new(Felt::new(5362129305222679805), Felt::new(0)),
        ),
        (
            "decoder.batch_flags.c1.binary",
            QuadFelt::new(Felt::new(7857195453682114326), Felt::new(0)),
        ),
        (
            "decoder.batch_flags.c2.binary",
            QuadFelt::new(Felt::new(7691051559149421836), Felt::new(0)),
        ),
        (
            "decoder.general.split_loop.s0.binary",
            QuadFelt::new(Felt::new(14496120396244092127), Felt::new(0)),
        ),
        (
            "decoder.general.dyn.h4.zero",
            QuadFelt::new(Felt::new(1277805081675897337), Felt::new(0)),
        ),
        (
            "decoder.general.dyn.h5.zero",
            QuadFelt::new(Felt::new(4194588350245381799), Felt::new(0)),
        ),
        (
            "decoder.general.dyn.h6.zero",
            QuadFelt::new(Felt::new(16022182314963541978), Felt::new(0)),
        ),
        (
            "decoder.general.dyn.h7.zero",
            QuadFelt::new(Felt::new(8836314757936512908), Felt::new(0)),
        ),
        (
            "decoder.general.repeat.s0.one",
            QuadFelt::new(Felt::new(12665553195229242113), Felt::new(0)),
        ),
        (
            "decoder.general.repeat.h4.one",
            QuadFelt::new(Felt::new(7110671376227656729), Felt::new(0)),
        ),
        (
            "decoder.general.end.loop.s0.zero",
            QuadFelt::new(Felt::new(17349561739015487668), Felt::new(0)),
        ),
        (
            "decoder.general.end_repeat.h0.carry",
            QuadFelt::new(Felt::new(14675084366068366020), Felt::new(0)),
        ),
        (
            "decoder.general.end_repeat.h1.carry",
            QuadFelt::new(Felt::new(7206936627190077403), Felt::new(0)),
        ),
        (
            "decoder.general.end_repeat.h2.carry",
            QuadFelt::new(Felt::new(6718740807857903289), Felt::new(0)),
        ),
        (
            "decoder.general.end_repeat.h3.carry",
            QuadFelt::new(Felt::new(17516850364483319430), Felt::new(0)),
        ),
        (
            "decoder.general.end_repeat.h4.carry",
            QuadFelt::new(Felt::new(6539200550348860466), Felt::new(0)),
        ),
        (
            "decoder.general.halt.next",
            QuadFelt::new(Felt::new(46417891308149319), Felt::new(0)),
        ),
        (
            "decoder.in_span.first_row",
            QuadFelt::new(Felt::new(14927496178105230921), Felt::new(0)),
        ),
        (
            "decoder.in_span.binary",
            QuadFelt::new(Felt::new(14486244054610710736), Felt::new(0)),
        ),
        (
            "decoder.in_span.span",
            QuadFelt::new(Felt::new(466300909996410452), Felt::new(0)),
        ),
        (
            "decoder.in_span.respan",
            QuadFelt::new(Felt::new(3338971954421326066), Felt::new(0)),
        ),
        (
            "decoder.group_count.delta.binary",
            QuadFelt::new(Felt::new(14515312709656548917), Felt::new(0)),
        ),
        (
            "decoder.group_count.decrement.h0_or_imm",
            QuadFelt::new(Felt::new(13182337539042779943), Felt::new(0)),
        ),
        (
            "decoder.group_count.span_decrement",
            QuadFelt::new(Felt::new(6058211846758132294), Felt::new(0)),
        ),
        (
            "decoder.group_count.end_or_respan.hold",
            QuadFelt::new(Felt::new(11052268645110095431), Felt::new(0)),
        ),
        (
            "decoder.group_count.end.zero",
            QuadFelt::new(Felt::new(8085923270334721350), Felt::new(0)),
        ),
        (
            "decoder.op_group.shift",
            QuadFelt::new(Felt::new(1312737539633457020), Felt::new(0)),
        ),
        (
            "decoder.op_group.end_or_respan.h0.zero",
            QuadFelt::new(Felt::new(12951763225475877068), Felt::new(0)),
        ),
        (
            "decoder.op_index.span_respan.reset",
            QuadFelt::new(Felt::new(10573491584444022281), Felt::new(0)),
        ),
        (
            "decoder.op_index.new_group.reset",
            QuadFelt::new(Felt::new(6175768744156945971), Felt::new(0)),
        ),
        (
            "decoder.op_index.increment",
            QuadFelt::new(Felt::new(11099022161747498050), Felt::new(0)),
        ),
        (
            "decoder.op_index.range",
            QuadFelt::new(Felt::new(10884671635123915786), Felt::new(0)),
        ),
        (
            "decoder.batch_flags.span_sum",
            QuadFelt::new(Felt::new(3694838697400308733), Felt::new(0)),
        ),
        (
            "decoder.batch_flags.zero_when_not_span",
            QuadFelt::new(Felt::new(3630764990867231714), Felt::new(0)),
        ),
        (
            "decoder.batch_flags.h4.zero",
            QuadFelt::new(Felt::new(2244382601531916648), Felt::new(0)),
        ),
        (
            "decoder.batch_flags.h5.zero",
            QuadFelt::new(Felt::new(15434877991581266285), Felt::new(0)),
        ),
        (
            "decoder.batch_flags.h6.zero",
            QuadFelt::new(Felt::new(7419023179375721027), Felt::new(0)),
        ),
        (
            "decoder.batch_flags.h7.zero",
            QuadFelt::new(Felt::new(7459745966287177285), Felt::new(0)),
        ),
        (
            "decoder.batch_flags.h2.zero",
            QuadFelt::new(Felt::new(11698744832781440772), Felt::new(0)),
        ),
        (
            "decoder.batch_flags.h3.zero",
            QuadFelt::new(Felt::new(8586259512688079232), Felt::new(0)),
        ),
        (
            "decoder.batch_flags.h1.zero",
            QuadFelt::new(Felt::new(7969602088154595265), Felt::new(0)),
        ),
        (
            "decoder.addr.hold_in_span",
            QuadFelt::new(Felt::new(5569758276797826136), Felt::new(0)),
        ),
        (
            "decoder.addr.respan.increment",
            QuadFelt::new(Felt::new(7010123233147094271), Felt::new(0)),
        ),
        (
            "decoder.addr.halt.zero",
            QuadFelt::new(Felt::new(571992094937652912), Felt::new(0)),
        ),
        (
            "decoder.control_flow.sp_complement",
            QuadFelt::new(Felt::new(2368373158779190039), Felt::new(0)),
        ),
        (
            "decoder.bus.p1.transition",
            QuadFelt::new(Felt::new(11611432650982424455), Felt::new(10377793451000863001)),
        ),
        (
            "decoder.bus.p2.transition",
            QuadFelt::new(Felt::new(15040597896341508305), Felt::new(11465419388996005277)),
        ),
        (
            "decoder.bus.p3.transition",
            QuadFelt::new(Felt::new(9395869302542898577), Felt::new(6472917827183803848)),
        ),
        (
            "stack.general.transition.0",
            QuadFelt::new(Felt::new(2617308096902219240), Felt::new(0)),
        ),
        (
            "stack.general.transition.1",
            QuadFelt::new(Felt::new(4439102810547612775), Felt::new(0)),
        ),
        (
            "stack.general.transition.2",
            QuadFelt::new(Felt::new(15221140463513662734), Felt::new(0)),
        ),
        (
            "stack.general.transition.3",
            QuadFelt::new(Felt::new(4910128267170087966), Felt::new(0)),
        ),
        (
            "stack.general.transition.4",
            QuadFelt::new(Felt::new(8221884229886405628), Felt::new(0)),
        ),
        (
            "stack.general.transition.5",
            QuadFelt::new(Felt::new(87491100192562680), Felt::new(0)),
        ),
        (
            "stack.general.transition.6",
            QuadFelt::new(Felt::new(11411892308848385202), Felt::new(0)),
        ),
        (
            "stack.general.transition.7",
            QuadFelt::new(Felt::new(2425094460891103256), Felt::new(0)),
        ),
        (
            "stack.general.transition.8",
            QuadFelt::new(Felt::new(2767534397043537043), Felt::new(0)),
        ),
        (
            "stack.general.transition.9",
            QuadFelt::new(Felt::new(11686523590994044007), Felt::new(0)),
        ),
        (
            "stack.general.transition.10",
            QuadFelt::new(Felt::new(15000969044032170777), Felt::new(0)),
        ),
        (
            "stack.general.transition.11",
            QuadFelt::new(Felt::new(17422355615541008592), Felt::new(0)),
        ),
        (
            "stack.general.transition.12",
            QuadFelt::new(Felt::new(2555448945580115158), Felt::new(0)),
        ),
        (
            "stack.general.transition.13",
            QuadFelt::new(Felt::new(8864896307613509), Felt::new(0)),
        ),
        (
            "stack.general.transition.14",
            QuadFelt::new(Felt::new(3997062422665481459), Felt::new(0)),
        ),
        (
            "stack.general.transition.15",
            QuadFelt::new(Felt::new(6149720027324442163), Felt::new(0)),
        ),
        (
            "stack.overflow.depth.first_row",
            QuadFelt::new(Felt::new(1820735510664294085), Felt::new(0)),
        ),
        (
            "stack.overflow.depth.last_row",
            QuadFelt::new(Felt::new(12520055704510454391), Felt::new(0)),
        ),
        (
            "stack.overflow.addr.first_row",
            QuadFelt::new(Felt::new(9235172344178625178), Felt::new(0)),
        ),
        (
            "stack.overflow.addr.last_row",
            QuadFelt::new(Felt::new(6001883085148683205), Felt::new(0)),
        ),
        (
            "stack.overflow.depth.transition",
            QuadFelt::new(Felt::new(6706883717633639596), Felt::new(0)),
        ),
        (
            "stack.overflow.flag.transition",
            QuadFelt::new(Felt::new(957822818659219409), Felt::new(0)),
        ),
        (
            "stack.overflow.addr.transition",
            QuadFelt::new(Felt::new(13739720401332236216), Felt::new(0)),
        ),
        (
            "stack.overflow.zero_insert.transition",
            QuadFelt::new(Felt::new(15830245309845547857), Felt::new(0)),
        ),
        (
            "stack.overflow.bus.transition",
            QuadFelt::new(Felt::new(7384164985445418427), Felt::new(3858806565449404456)),
        ),
        ("stack.ops.pad", QuadFelt::new(Felt::new(13331629930659656176), Felt::new(0))),
        ("stack.ops.dup", QuadFelt::new(Felt::new(756650319667756050), Felt::new(0))),
        ("stack.ops.dup", QuadFelt::new(Felt::new(8866275161884692697), Felt::new(0))),
        ("stack.ops.dup", QuadFelt::new(Felt::new(3836534398031583164), Felt::new(0))),
        ("stack.ops.dup", QuadFelt::new(Felt::new(14027345575708861734), Felt::new(0))),
        ("stack.ops.dup", QuadFelt::new(Felt::new(6758311777121484896), Felt::new(0))),
        ("stack.ops.dup", QuadFelt::new(Felt::new(3070735592903657788), Felt::new(0))),
        ("stack.ops.dup", QuadFelt::new(Felt::new(7754656097784875208), Felt::new(0))),
        ("stack.ops.dup", QuadFelt::new(Felt::new(6720121361576140513), Felt::new(0))),
        ("stack.ops.dup", QuadFelt::new(Felt::new(17539764796672551158), Felt::new(0))),
        ("stack.ops.dup", QuadFelt::new(Felt::new(10804911883091000860), Felt::new(0))),
        ("stack.ops.dup", QuadFelt::new(Felt::new(9611708950007293491), Felt::new(0))),
        ("stack.ops.dup", QuadFelt::new(Felt::new(8853070398648442411), Felt::new(0))),
        ("stack.ops.clk", QuadFelt::new(Felt::new(9109734313690111543), Felt::new(0))),
        ("stack.ops.swap", QuadFelt::new(Felt::new(3018402783504114630), Felt::new(0))),
        ("stack.ops.swap", QuadFelt::new(Felt::new(17272825861332302734), Felt::new(0))),
        ("stack.ops.movup", QuadFelt::new(Felt::new(6365383181668196029), Felt::new(0))),
        ("stack.ops.movup", QuadFelt::new(Felt::new(11479712264864576587), Felt::new(0))),
        ("stack.ops.movup", QuadFelt::new(Felt::new(12050324136647260589), Felt::new(0))),
        ("stack.ops.movup", QuadFelt::new(Felt::new(4842889514271599822), Felt::new(0))),
        ("stack.ops.movup", QuadFelt::new(Felt::new(7388624400246275858), Felt::new(0))),
        ("stack.ops.movup", QuadFelt::new(Felt::new(10382124953564405655), Felt::new(0))),
        ("stack.ops.movup", QuadFelt::new(Felt::new(14668661130070444298), Felt::new(0))),
        ("stack.ops.movdn", QuadFelt::new(Felt::new(7617911967740804399), Felt::new(0))),
        ("stack.ops.movdn", QuadFelt::new(Felt::new(10587498815844952065), Felt::new(0))),
        ("stack.ops.movdn", QuadFelt::new(Felt::new(6234074065813353677), Felt::new(0))),
        ("stack.ops.movdn", QuadFelt::new(Felt::new(8228745571736556881), Felt::new(0))),
        ("stack.ops.movdn", QuadFelt::new(Felt::new(1255130201489737978), Felt::new(0))),
        ("stack.ops.movdn", QuadFelt::new(Felt::new(4861541115171604729), Felt::new(0))),
        ("stack.ops.movdn", QuadFelt::new(Felt::new(7218300239612772413), Felt::new(0))),
        ("stack.ops.swapw", QuadFelt::new(Felt::new(1397391365707566947), Felt::new(0))),
        ("stack.ops.swapw", QuadFelt::new(Felt::new(15192275354424729852), Felt::new(0))),
        ("stack.ops.swapw", QuadFelt::new(Felt::new(8991791753517007572), Felt::new(0))),
        ("stack.ops.swapw", QuadFelt::new(Felt::new(6845904526592099338), Felt::new(0))),
        ("stack.ops.swapw", QuadFelt::new(Felt::new(14405008868848810993), Felt::new(0))),
        ("stack.ops.swapw", QuadFelt::new(Felt::new(14818059880037013402), Felt::new(0))),
        ("stack.ops.swapw", QuadFelt::new(Felt::new(12858781526955010288), Felt::new(0))),
        ("stack.ops.swapw", QuadFelt::new(Felt::new(4346525868099676574), Felt::new(0))),
        ("stack.ops.swapw2", QuadFelt::new(Felt::new(12020803221700843056), Felt::new(0))),
        ("stack.ops.swapw2", QuadFelt::new(Felt::new(5905514554571101818), Felt::new(0))),
        ("stack.ops.swapw2", QuadFelt::new(Felt::new(13967530246007855218), Felt::new(0))),
        ("stack.ops.swapw2", QuadFelt::new(Felt::new(1745280905200466463), Felt::new(0))),
        ("stack.ops.swapw2", QuadFelt::new(Felt::new(8273384627661819419), Felt::new(0))),
        ("stack.ops.swapw2", QuadFelt::new(Felt::new(17907212562142949954), Felt::new(0))),
        ("stack.ops.swapw2", QuadFelt::new(Felt::new(10641837676859047674), Felt::new(0))),
        ("stack.ops.swapw2", QuadFelt::new(Felt::new(5696399439164028901), Felt::new(0))),
        ("stack.ops.swapw3", QuadFelt::new(Felt::new(261758456050090541), Felt::new(0))),
        ("stack.ops.swapw3", QuadFelt::new(Felt::new(13783565204182644984), Felt::new(0))),
        ("stack.ops.swapw3", QuadFelt::new(Felt::new(8373199292442046895), Felt::new(0))),
        ("stack.ops.swapw3", QuadFelt::new(Felt::new(17987956356814792948), Felt::new(0))),
        ("stack.ops.swapw3", QuadFelt::new(Felt::new(15863165148623313437), Felt::new(0))),
        ("stack.ops.swapw3", QuadFelt::new(Felt::new(15873554387396407564), Felt::new(0))),
        ("stack.ops.swapw3", QuadFelt::new(Felt::new(13572800254923888612), Felt::new(0))),
        ("stack.ops.swapw3", QuadFelt::new(Felt::new(37494485778659889), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(5468305410596890575), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(8148573700621797018), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(174223531403505930), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(7472429897136677074), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(9085995615849733227), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(17751305329307070351), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(12464875440922891257), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(7381981033510767101), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(14206386269299463916), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(5165712881513112310), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(9505024677507267655), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(7199235098885318815), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(14863071265127885763), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(7964997496183729586), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(17447611484236572336), Felt::new(0))),
        ("stack.ops.swapdw", QuadFelt::new(Felt::new(7663698430658282360), Felt::new(0))),
        ("stack.ops.cswap", QuadFelt::new(Felt::new(4142872517165003079), Felt::new(0))),
        ("stack.ops.cswap", QuadFelt::new(Felt::new(18107469477286194402), Felt::new(0))),
        ("stack.ops.cswap", QuadFelt::new(Felt::new(8228755909294702214), Felt::new(0))),
        ("stack.ops.cswapw", QuadFelt::new(Felt::new(7382517392819628451), Felt::new(0))),
        ("stack.ops.cswapw", QuadFelt::new(Felt::new(4827417633003237585), Felt::new(0))),
        ("stack.ops.cswapw", QuadFelt::new(Felt::new(17779390882653606052), Felt::new(0))),
        ("stack.ops.cswapw", QuadFelt::new(Felt::new(16587491652407655425), Felt::new(0))),
        ("stack.ops.cswapw", QuadFelt::new(Felt::new(6936098212561125534), Felt::new(0))),
        ("stack.ops.cswapw", QuadFelt::new(Felt::new(5094958697700743127), Felt::new(0))),
        ("stack.ops.cswapw", QuadFelt::new(Felt::new(189412762651021203), Felt::new(0))),
        ("stack.ops.cswapw", QuadFelt::new(Felt::new(8308993958309806023), Felt::new(0))),
        ("stack.ops.cswapw", QuadFelt::new(Felt::new(16580384734945713908), Felt::new(0))),
        (
            "stack.system.assert",
            QuadFelt::new(Felt::new(13682615670963393420), Felt::new(0)),
        ),
        (
            "stack.system.caller",
            QuadFelt::new(Felt::new(16674981897661760210), Felt::new(0)),
        ),
        (
            "stack.system.caller",
            QuadFelt::new(Felt::new(14361028107722480662), Felt::new(0)),
        ),
        (
            "stack.system.caller",
            QuadFelt::new(Felt::new(9738252875195915138), Felt::new(0)),
        ),
        (
            "stack.system.caller",
            QuadFelt::new(Felt::new(15161342143096572193), Felt::new(0)),
        ),
        ("stack.io.sdepth", QuadFelt::new(Felt::new(9690568048381717864), Felt::new(0))),
        (
            "stack.field.add.transition",
            QuadFelt::new(Felt::new(12162183238628940886), Felt::new(0)),
        ),
        (
            "stack.field.neg.transition",
            QuadFelt::new(Felt::new(5581128975715924145), Felt::new(0)),
        ),
        (
            "stack.field.mul.transition",
            QuadFelt::new(Felt::new(8554389406737436796), Felt::new(0)),
        ),
        (
            "stack.field.inv.transition",
            QuadFelt::new(Felt::new(5063887741998958642), Felt::new(0)),
        ),
        (
            "stack.field.incr.transition",
            QuadFelt::new(Felt::new(4639763508506743987), Felt::new(0)),
        ),
        (
            "stack.field.not.binary",
            QuadFelt::new(Felt::new(2252712733428984777), Felt::new(0)),
        ),
        (
            "stack.field.not.result",
            QuadFelt::new(Felt::new(13177116281714227608), Felt::new(0)),
        ),
        (
            "stack.field.and.binary.a",
            QuadFelt::new(Felt::new(14850227941716122627), Felt::new(0)),
        ),
        (
            "stack.field.and.binary.b",
            QuadFelt::new(Felt::new(18175567784024467771), Felt::new(0)),
        ),
        (
            "stack.field.and.result",
            QuadFelt::new(Felt::new(2412459788431241921), Felt::new(0)),
        ),
        (
            "stack.field.or.binary.a",
            QuadFelt::new(Felt::new(8597905655955046372), Felt::new(0)),
        ),
        (
            "stack.field.or.binary.b",
            QuadFelt::new(Felt::new(1482536575467916010), Felt::new(0)),
        ),
        (
            "stack.field.or.result",
            QuadFelt::new(Felt::new(8108995209839065445), Felt::new(0)),
        ),
        (
            "stack.field.eq.zero_product",
            QuadFelt::new(Felt::new(14385477599012828093), Felt::new(0)),
        ),
        (
            "stack.field.eq.result",
            QuadFelt::new(Felt::new(17777414138310332081), Felt::new(0)),
        ),
        (
            "stack.field.eqz.zero_product",
            QuadFelt::new(Felt::new(1585550152724577414), Felt::new(0)),
        ),
        (
            "stack.field.eqz.result",
            QuadFelt::new(Felt::new(8914500005861323211), Felt::new(0)),
        ),
        (
            "stack.field.expacc.square",
            QuadFelt::new(Felt::new(10821948734140194924), Felt::new(0)),
        ),
        (
            "stack.field.expacc.val",
            QuadFelt::new(Felt::new(18138155306684050585), Felt::new(0)),
        ),
        (
            "stack.field.expacc.acc",
            QuadFelt::new(Felt::new(11221181362690184920), Felt::new(0)),
        ),
        (
            "stack.field.expacc.shift",
            QuadFelt::new(Felt::new(9522158304362954603), Felt::new(0)),
        ),
        (
            "stack.field.expacc.bit_binary",
            QuadFelt::new(Felt::new(13901486800460794091), Felt::new(0)),
        ),
        (
            "stack.field.ext2mul.d0",
            QuadFelt::new(Felt::new(7911065669822474568), Felt::new(0)),
        ),
        (
            "stack.field.ext2mul.d1",
            QuadFelt::new(Felt::new(3598619357058098113), Felt::new(0)),
        ),
        (
            "stack.field.ext2mul.c0",
            QuadFelt::new(Felt::new(996509971607279275), Felt::new(0)),
        ),
        (
            "stack.field.ext2mul.c1",
            QuadFelt::new(Felt::new(13600174711341153155), Felt::new(0)),
        ),
        (
            "stack.u32.validity",
            QuadFelt::new(Felt::new(3644067674547670918), Felt::new(0)),
        ),
        (
            "stack.u32.limbs.lo",
            QuadFelt::new(Felt::new(13697666666561534208), Felt::new(0)),
        ),
        (
            "stack.u32.limbs.hi",
            QuadFelt::new(Felt::new(18192033048549134928), Felt::new(0)),
        ),
        ("stack.u32.split", QuadFelt::new(Felt::new(15525883193297412094), Felt::new(0))),
        ("stack.u32.add", QuadFelt::new(Felt::new(7349446320624523226), Felt::new(0))),
        ("stack.u32.add3", QuadFelt::new(Felt::new(16762578733999451576), Felt::new(0))),
        ("stack.u32.sub.agg", QuadFelt::new(Felt::new(3623920048867462270), Felt::new(0))),
        (
            "stack.u32.sub.borrow",
            QuadFelt::new(Felt::new(5319755831391255333), Felt::new(0)),
        ),
        (
            "stack.u32.sub.diff",
            QuadFelt::new(Felt::new(15655064156696355905), Felt::new(0)),
        ),
        ("stack.u32.mul", QuadFelt::new(Felt::new(6752318701651577568), Felt::new(0))),
        ("stack.u32.madd", QuadFelt::new(Felt::new(2748176389622229846), Felt::new(0))),
        ("stack.u32.div.agg", QuadFelt::new(Felt::new(40380517428295215), Felt::new(0))),
        (
            "stack.u32.div.quot_bound",
            QuadFelt::new(Felt::new(15350419499859122435), Felt::new(0)),
        ),
        (
            "stack.u32.div.rem_bound",
            QuadFelt::new(Felt::new(14341334560480736404), Felt::new(0)),
        ),
        (
            "stack.u32.assert2.b",
            QuadFelt::new(Felt::new(6089961092604608348), Felt::new(0)),
        ),
        (
            "stack.u32.assert2.a",
            QuadFelt::new(Felt::new(3427128116590576361), Felt::new(0)),
        ),
        (
            "stack.crypto.cryptostream.capacity.0",
            QuadFelt::new(Felt::new(12685385640397555155), Felt::new(0)),
        ),
        (
            "stack.crypto.cryptostream.capacity.1",
            QuadFelt::new(Felt::new(17365149299857381549), Felt::new(0)),
        ),
        (
            "stack.crypto.cryptostream.capacity.2",
            QuadFelt::new(Felt::new(7455833729327549495), Felt::new(0)),
        ),
        (
            "stack.crypto.cryptostream.capacity.3",
            QuadFelt::new(Felt::new(15687115573708323478), Felt::new(0)),
        ),
        (
            "stack.crypto.cryptostream.ptr.src",
            QuadFelt::new(Felt::new(7143356749732107964), Felt::new(0)),
        ),
        (
            "stack.crypto.cryptostream.ptr.dst",
            QuadFelt::new(Felt::new(16804762938330714938), Felt::new(0)),
        ),
        (
            "stack.crypto.cryptostream.tail.0",
            QuadFelt::new(Felt::new(11562801811268566657), Felt::new(0)),
        ),
        (
            "stack.crypto.cryptostream.tail.1",
            QuadFelt::new(Felt::new(6374246579471617400), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.0",
            QuadFelt::new(Felt::new(6682735393816016083), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.1",
            QuadFelt::new(Felt::new(15946014808270501272), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.2",
            QuadFelt::new(Felt::new(15603944589931385962), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.3",
            QuadFelt::new(Felt::new(9275882712531701258), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.4",
            QuadFelt::new(Felt::new(2477075229563534723), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.5",
            QuadFelt::new(Felt::new(5290505604769958968), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.6",
            QuadFelt::new(Felt::new(2851265439044985455), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.7",
            QuadFelt::new(Felt::new(18383212236849004064), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.8",
            QuadFelt::new(Felt::new(1727422736811819477), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.9",
            QuadFelt::new(Felt::new(8661298711862814846), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.10",
            QuadFelt::new(Felt::new(4909615103768362856), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.11",
            QuadFelt::new(Felt::new(6313538606129191078), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.12",
            QuadFelt::new(Felt::new(16477933543947236322), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.stack.13",
            QuadFelt::new(Felt::new(8923348207341089911), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.tmp0.0",
            QuadFelt::new(Felt::new(12031819951881691966), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.tmp0.1",
            QuadFelt::new(Felt::new(16763623523728809700), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.tmp1.0",
            QuadFelt::new(Felt::new(16448780392400843578), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.tmp1.1",
            QuadFelt::new(Felt::new(13659290005693061581), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.acc.0",
            QuadFelt::new(Felt::new(9982904041042376807), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerbase.acc.1",
            QuadFelt::new(Felt::new(5949627607219451329), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.0",
            QuadFelt::new(Felt::new(4258650708569289369), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.1",
            QuadFelt::new(Felt::new(10623987720748853996), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.2",
            QuadFelt::new(Felt::new(7214338718283715042), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.3",
            QuadFelt::new(Felt::new(11353293984106841353), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.4",
            QuadFelt::new(Felt::new(13021994910061529075), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.5",
            QuadFelt::new(Felt::new(16890098475354732519), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.6",
            QuadFelt::new(Felt::new(17909680271515252883), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.7",
            QuadFelt::new(Felt::new(17436574006020893038), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.8",
            QuadFelt::new(Felt::new(11510839286135128168), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.9",
            QuadFelt::new(Felt::new(5781748113887851533), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.10",
            QuadFelt::new(Felt::new(14599010851776253883), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.11",
            QuadFelt::new(Felt::new(9495625123030210045), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.12",
            QuadFelt::new(Felt::new(7672904073310511358), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.stack.13",
            QuadFelt::new(Felt::new(775511618954631186), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.tmp.0",
            QuadFelt::new(Felt::new(2516572400933728962), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.tmp.1",
            QuadFelt::new(Felt::new(16462328690369258086), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.acc.0",
            QuadFelt::new(Felt::new(4231043957658294146), Felt::new(0)),
        ),
        (
            "stack.crypto.hornerext.acc.1",
            QuadFelt::new(Felt::new(16476104241930761470), Felt::new(0)),
        ),
        (
            "range.main.v.first_row",
            QuadFelt::new(Felt::new(1112338059331632069), Felt::new(0)),
        ),
        (
            "range.main.v.last_row",
            QuadFelt::new(Felt::new(13352757668188868927), Felt::new(0)),
        ),
        (
            "range.main.v.transition",
            QuadFelt::new(Felt::new(12797082443503681195), Felt::new(0)),
        ),
        (
            "range.bus.transition",
            QuadFelt::new(Felt::new(10365289165200035540), Felt::new(16469718665506609592)),
        ),
        (
            "chiplets.selectors.s0.binary",
            QuadFelt::new(Felt::new(13339369523717109295), Felt::new(0)),
        ),
        (
            "chiplets.selectors.s1.binary",
            QuadFelt::new(Felt::new(5399081030326323264), Felt::new(0)),
        ),
        (
            "chiplets.selectors.s2.binary",
            QuadFelt::new(Felt::new(12423271937388024622), Felt::new(0)),
        ),
        (
            "chiplets.selectors.s3.binary",
            QuadFelt::new(Felt::new(6104289749728022881), Felt::new(0)),
        ),
        (
            "chiplets.selectors.s4.binary",
            QuadFelt::new(Felt::new(1241452016395320053), Felt::new(0)),
        ),
        (
            "chiplets.selectors.s0.stability",
            QuadFelt::new(Felt::new(14729701512419667041), Felt::new(0)),
        ),
        (
            "chiplets.selectors.s1.stability",
            QuadFelt::new(Felt::new(8909164618174988456), Felt::new(0)),
        ),
        (
            "chiplets.selectors.s2.stability",
            QuadFelt::new(Felt::new(17247285692427399965), Felt::new(0)),
        ),
        (
            "chiplets.selectors.s3.stability",
            QuadFelt::new(Felt::new(18202363660063267394), Felt::new(0)),
        ),
        (
            "chiplets.selectors.s4.stability",
            QuadFelt::new(Felt::new(6610185862666795331), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.init",
            QuadFelt::new(Felt::new(4099164774137728313), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.init",
            QuadFelt::new(Felt::new(14454658395404025559), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.init",
            QuadFelt::new(Felt::new(14045750606291612344), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.init",
            QuadFelt::new(Felt::new(4962577616206122596), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.init",
            QuadFelt::new(Felt::new(17693281290536116739), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.init",
            QuadFelt::new(Felt::new(2566307954601069485), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.init",
            QuadFelt::new(Felt::new(7014825251917345868), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.init",
            QuadFelt::new(Felt::new(12643485402937350728), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.init",
            QuadFelt::new(Felt::new(13773518984372045426), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.init",
            QuadFelt::new(Felt::new(12725128195760136818), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.init",
            QuadFelt::new(Felt::new(12789485480014529422), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.init",
            QuadFelt::new(Felt::new(2064555680622899263), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.external",
            QuadFelt::new(Felt::new(14965667877036435255), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.external",
            QuadFelt::new(Felt::new(8649163745447702544), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.external",
            QuadFelt::new(Felt::new(1871614405591673138), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.external",
            QuadFelt::new(Felt::new(3904379054162154278), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.external",
            QuadFelt::new(Felt::new(14269032524621289009), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.external",
            QuadFelt::new(Felt::new(819271014970897034), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.external",
            QuadFelt::new(Felt::new(7445347300254257143), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.external",
            QuadFelt::new(Felt::new(18264241495848405505), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.external",
            QuadFelt::new(Felt::new(4022419953426403986), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.external",
            QuadFelt::new(Felt::new(16556927457681327599), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.external",
            QuadFelt::new(Felt::new(7630095839895141032), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.external",
            QuadFelt::new(Felt::new(17073915086247210099), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.internal",
            QuadFelt::new(Felt::new(12354414453280530108), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.internal",
            QuadFelt::new(Felt::new(15162996181146864722), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.internal",
            QuadFelt::new(Felt::new(3799802584325483032), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.internal",
            QuadFelt::new(Felt::new(7503666179952416559), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.internal",
            QuadFelt::new(Felt::new(5195105242383400264), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.internal",
            QuadFelt::new(Felt::new(2672184473444251603), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.internal",
            QuadFelt::new(Felt::new(1545887997090789749), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.internal",
            QuadFelt::new(Felt::new(212978569305465235), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.internal",
            QuadFelt::new(Felt::new(7828622218965306157), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.internal",
            QuadFelt::new(Felt::new(15313472288808367905), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.internal",
            QuadFelt::new(Felt::new(1544567903496578828), Felt::new(0)),
        ),
        (
            "chiplets.hasher.permutation.internal",
            QuadFelt::new(Felt::new(6950222684787745222), Felt::new(0)),
        ),
        (
            "chiplets.hasher.selectors.binary",
            QuadFelt::new(Felt::new(14125970682131069043), Felt::new(0)),
        ),
        (
            "chiplets.hasher.selectors.binary",
            QuadFelt::new(Felt::new(15397757255883077151), Felt::new(0)),
        ),
        (
            "chiplets.hasher.selectors.binary",
            QuadFelt::new(Felt::new(8206822507230624215), Felt::new(0)),
        ),
        (
            "chiplets.hasher.selectors.stability",
            QuadFelt::new(Felt::new(6479352521221779748), Felt::new(0)),
        ),
        (
            "chiplets.hasher.selectors.stability",
            QuadFelt::new(Felt::new(210512107158412246), Felt::new(0)),
        ),
        (
            "chiplets.hasher.selectors.continuation",
            QuadFelt::new(Felt::new(15715623607701353500), Felt::new(0)),
        ),
        (
            "chiplets.hasher.selectors.invalid",
            QuadFelt::new(Felt::new(13405975868755856198), Felt::new(0)),
        ),
        (
            "chiplets.hasher.abp.capacity",
            QuadFelt::new(Felt::new(1899048554833837921), Felt::new(0)),
        ),
        (
            "chiplets.hasher.abp.capacity",
            QuadFelt::new(Felt::new(12437244811220538952), Felt::new(0)),
        ),
        (
            "chiplets.hasher.abp.capacity",
            QuadFelt::new(Felt::new(10139411892992277524), Felt::new(0)),
        ),
        (
            "chiplets.hasher.abp.capacity",
            QuadFelt::new(Felt::new(15737189571305100043), Felt::new(0)),
        ),
        (
            "chiplets.hasher.output.index",
            QuadFelt::new(Felt::new(16881983815653283540), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.index.binary",
            QuadFelt::new(Felt::new(13143318647927561805), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.index.stability",
            QuadFelt::new(Felt::new(13941205471766075471), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.index.output",
            QuadFelt::new(Felt::new(5408022086168955043), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.capacity",
            QuadFelt::new(Felt::new(5486621160745001548), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.capacity",
            QuadFelt::new(Felt::new(15134096900702524735), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.capacity",
            QuadFelt::new(Felt::new(17576701782509621064), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.capacity",
            QuadFelt::new(Felt::new(14561117990313963177), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.digest.rate0",
            QuadFelt::new(Felt::new(8725087617587926550), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.digest.rate0",
            QuadFelt::new(Felt::new(12515088494833138961), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.digest.rate0",
            QuadFelt::new(Felt::new(6965398204541535664), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.digest.rate0",
            QuadFelt::new(Felt::new(11433229023120737569), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.digest.rate1",
            QuadFelt::new(Felt::new(4884394573065500633), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.digest.rate1",
            QuadFelt::new(Felt::new(16805257998005939804), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.digest.rate1",
            QuadFelt::new(Felt::new(55029178798193706), Felt::new(0)),
        ),
        (
            "chiplets.hasher.merkle.digest.rate1",
            QuadFelt::new(Felt::new(17219870521967201546), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.op.binary",
            QuadFelt::new(Felt::new(15474882094825938582), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.op.stability",
            QuadFelt::new(Felt::new(654157308496499224), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.a_bits.binary",
            QuadFelt::new(Felt::new(1034651076143976640), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.a_bits.binary",
            QuadFelt::new(Felt::new(4003142075320695647), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.a_bits.binary",
            QuadFelt::new(Felt::new(303909215511455897), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.a_bits.binary",
            QuadFelt::new(Felt::new(5362728732691526694), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.b_bits.binary",
            QuadFelt::new(Felt::new(11650758097842027858), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.b_bits.binary",
            QuadFelt::new(Felt::new(7007196355931725843), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.b_bits.binary",
            QuadFelt::new(Felt::new(11561896106611918266), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.b_bits.binary",
            QuadFelt::new(Felt::new(4060803575635852640), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.first_row",
            QuadFelt::new(Felt::new(14840226897639012209), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.first_row",
            QuadFelt::new(Felt::new(15513879563001185502), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.first_row",
            QuadFelt::new(Felt::new(5652235828559265944), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.input.transition",
            QuadFelt::new(Felt::new(12380774213272670463), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.input.transition",
            QuadFelt::new(Felt::new(7940993120185857575), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.output.prev",
            QuadFelt::new(Felt::new(7984380758996607942), Felt::new(0)),
        ),
        (
            "chiplets.bitwise.output.aggregate",
            QuadFelt::new(Felt::new(11338003127856250266), Felt::new(0)),
        ),
        (
            "chiplets.memory.binary",
            QuadFelt::new(Felt::new(6518050416979602887), Felt::new(0)),
        ),
        (
            "chiplets.memory.binary",
            QuadFelt::new(Felt::new(5143376107998730535), Felt::new(0)),
        ),
        (
            "chiplets.memory.binary",
            QuadFelt::new(Felt::new(1931814968789928617), Felt::new(0)),
        ),
        (
            "chiplets.memory.binary",
            QuadFelt::new(Felt::new(15470079227779320896), Felt::new(0)),
        ),
        (
            "chiplets.memory.word_idx.zero",
            QuadFelt::new(Felt::new(15459815149111314868), Felt::new(0)),
        ),
        (
            "chiplets.memory.word_idx.zero",
            QuadFelt::new(Felt::new(15800347411094406640), Felt::new(0)),
        ),
        (
            "chiplets.memory.first_row.zero",
            QuadFelt::new(Felt::new(16535341688150290637), Felt::new(0)),
        ),
        (
            "chiplets.memory.first_row.zero",
            QuadFelt::new(Felt::new(10335801429662869046), Felt::new(0)),
        ),
        (
            "chiplets.memory.first_row.zero",
            QuadFelt::new(Felt::new(17069212044771710732), Felt::new(0)),
        ),
        (
            "chiplets.memory.first_row.zero",
            QuadFelt::new(Felt::new(2325691270454543127), Felt::new(0)),
        ),
        (
            "chiplets.memory.delta.inv",
            QuadFelt::new(Felt::new(3175424288859001789), Felt::new(0)),
        ),
        (
            "chiplets.memory.delta.inv",
            QuadFelt::new(Felt::new(2653406619128719065), Felt::new(0)),
        ),
        (
            "chiplets.memory.delta.inv",
            QuadFelt::new(Felt::new(17858142172042463544), Felt::new(0)),
        ),
        (
            "chiplets.memory.delta.inv",
            QuadFelt::new(Felt::new(6206863499132972446), Felt::new(0)),
        ),
        (
            "chiplets.memory.delta.transition",
            QuadFelt::new(Felt::new(5078351230126014060), Felt::new(0)),
        ),
        (
            "chiplets.memory.scw.flag",
            QuadFelt::new(Felt::new(18433800756547531428), Felt::new(0)),
        ),
        (
            "chiplets.memory.scw.reads",
            QuadFelt::new(Felt::new(9860250070209399141), Felt::new(0)),
        ),
        (
            "chiplets.memory.value.consistency",
            QuadFelt::new(Felt::new(11685142466069024125), Felt::new(0)),
        ),
        (
            "chiplets.memory.value.consistency",
            QuadFelt::new(Felt::new(15197055428524072106), Felt::new(0)),
        ),
        (
            "chiplets.memory.value.consistency",
            QuadFelt::new(Felt::new(14617718835619740558), Felt::new(0)),
        ),
        (
            "chiplets.memory.value.consistency",
            QuadFelt::new(Felt::new(12293856690108503135), Felt::new(0)),
        ),
        (
            "chiplets.ace.selector.binary",
            QuadFelt::new(Felt::new(2923257613600653893), Felt::new(0)),
        ),
        (
            "chiplets.ace.selector.binary",
            QuadFelt::new(Felt::new(4182752542556273997), Felt::new(0)),
        ),
        (
            "chiplets.ace.section.flags",
            QuadFelt::new(Felt::new(6988234832692930146), Felt::new(0)),
        ),
        (
            "chiplets.ace.section.flags",
            QuadFelt::new(Felt::new(835405595669725766), Felt::new(0)),
        ),
        (
            "chiplets.ace.section.flags",
            QuadFelt::new(Felt::new(17586531527103856415), Felt::new(0)),
        ),
        (
            "chiplets.ace.section.flags",
            QuadFelt::new(Felt::new(17554338302334456122), Felt::new(0)),
        ),
        (
            "chiplets.ace.section.flags",
            QuadFelt::new(Felt::new(7430977299237244825), Felt::new(0)),
        ),
        (
            "chiplets.ace.section.transition",
            QuadFelt::new(Felt::new(9634147153406944231), Felt::new(0)),
        ),
        (
            "chiplets.ace.section.transition",
            QuadFelt::new(Felt::new(3218972305890399047), Felt::new(0)),
        ),
        (
            "chiplets.ace.section.transition",
            QuadFelt::new(Felt::new(13940329983080013930), Felt::new(0)),
        ),
        (
            "chiplets.ace.section.transition",
            QuadFelt::new(Felt::new(10279516906957804027), Felt::new(0)),
        ),
        (
            "chiplets.ace.read.ids",
            QuadFelt::new(Felt::new(12585176929173957399), Felt::new(0)),
        ),
        (
            "chiplets.ace.read.to_eval",
            QuadFelt::new(Felt::new(30354383937781757), Felt::new(0)),
        ),
        (
            "chiplets.ace.eval.op",
            QuadFelt::new(Felt::new(12481984196006840571), Felt::new(0)),
        ),
        (
            "chiplets.ace.eval.result",
            QuadFelt::new(Felt::new(10009759308289170950), Felt::new(0)),
        ),
        (
            "chiplets.ace.eval.result",
            QuadFelt::new(Felt::new(9663557940632289707), Felt::new(0)),
        ),
        (
            "chiplets.ace.final.zero",
            QuadFelt::new(Felt::new(13957751954200526468), Felt::new(0)),
        ),
        (
            "chiplets.ace.final.zero",
            QuadFelt::new(Felt::new(13589615335587828352), Felt::new(0)),
        ),
        (
            "chiplets.ace.final.zero",
            QuadFelt::new(Felt::new(6818409555600730615), Felt::new(0)),
        ),
        (
            "chiplets.ace.first_row.start",
            QuadFelt::new(Felt::new(613969461051885369), Felt::new(0)),
        ),
        (
            "chiplets.kernel_rom.sfirst.binary",
            QuadFelt::new(Felt::new(9960038227923904827), Felt::new(0)),
        ),
        (
            "chiplets.kernel_rom.digest.contiguity",
            QuadFelt::new(Felt::new(12113043600978981430), Felt::new(0)),
        ),
        (
            "chiplets.kernel_rom.digest.contiguity",
            QuadFelt::new(Felt::new(15559322172686928295), Felt::new(0)),
        ),
        (
            "chiplets.kernel_rom.digest.contiguity",
            QuadFelt::new(Felt::new(12593211604980696045), Felt::new(0)),
        ),
        (
            "chiplets.kernel_rom.digest.contiguity",
            QuadFelt::new(Felt::new(4420066076215265302), Felt::new(0)),
        ),
        (
            "chiplets.kernel_rom.first_row.start",
            QuadFelt::new(Felt::new(3652575802134874675), Felt::new(0)),
        ),
        (
            "chiplets.bus.hash_kernel.transition",
            QuadFelt::new(Felt::new(8807035493766735820), Felt::new(5416084083202549081)),
        ),
        (
            "chiplets.bus.chiplets.transition",
            QuadFelt::new(Felt::new(14696969614088113799), Felt::new(1398723567985082498)),
        ),
        (
            "chiplets.bus.wiring.transition",
            QuadFelt::new(Felt::new(7613678356270986878), Felt::new(10445474671979834467)),
        ),
    ]
}

fn run_miden_vm_minimal_ood_evals_match() {
    let expected = expected_ood_evals();
    let air_path = load_miden_vm_minimal_air().expect("unable to read MidenVM_Minimal AIR");
    let air = generate_air(&air_path);
    let (_, stack_bus) = air
        .buses
        .iter()
        .find(|(name, _)| name.as_str() == "bus_3_stack_p1")
        .expect("missing stack overflow bus");
    assert_eq!(stack_bus.constraint_form, BusConstraintForm::Sum);
    let tagged = collect_tagged_constraints(&air);
    let ctx = OodContext::new(&air, OOD_SEED);
    let actual_values = eval_constraints_in_id_order(&air, &ctx);
    let actual: Vec<(&'static str, QuadFelt)> = CONSTRAINT_NAMES
        .iter()
        .copied()
        .zip(actual_values.into_iter().map(|value| value.to_quad()))
        .collect();
    assert_eq!(actual.len(), expected.len());
    for (idx, ((actual_name, actual_value), (expected_name, expected_value))) in
        actual.iter().zip(expected.iter()).enumerate()
    {
        assert_eq!(actual_name, expected_name);
        if actual_value != expected_value {
            let [a0, a1] = actual_value.to_base_elements();
            let [e0, e1] = expected_value.to_base_elements();
            let (tag, segment, domain) = tagged[idx];
            eprintln!(
                "mismatch at #{idx} tag={tag} {segment:?}/{domain:?} {actual_name}: actual={}+{}X expected={}+{}X",
                a0.as_int(),
                a1.as_int(),
                e0.as_int(),
                e1.as_int()
            );
            if *actual_name == "chiplets.bus.chiplets.transition" {
                use std::collections::BTreeMap;
                let debug_bus6 = std::env::var("AIRSCRIPT_DEBUG_BUS6").is_ok();
                let debug_verbose = std::env::var("AIRSCRIPT_DEBUG_BUS6_VERBOSE").is_ok();

                let (_bus_index, (_, chiplets_bus)) = air
                    .buses
                    .iter()
                    .enumerate()
                    .find(|(_, (name, _))| name.as_str() == "bus_6_chiplets_bus")
                    .expect("missing chiplets bus");

                let mut latch_counts: BTreeMap<air_ir::NodeIndex, usize> = BTreeMap::new();
                for bus_op in &chiplets_bus.bus_ops {
                    *latch_counts.entry(bus_op.latch).or_insert(0) += 1;
                }
                let multi_groups = latch_counts.values().filter(|count| **count > 1).count();
                eprintln!(
                    "debug bus_6 latch groups: total_ops={} groups={} multi_groups={}",
                    chiplets_bus.bus_ops.len(),
                    latch_counts.len(),
                    multi_groups,
                );
                for (latch, count) in latch_counts.iter().filter(|(_, c)| **c > 1) {
                    eprintln!("  latch {:?} -> ops {}", latch, count);
                }

                let graph = air.constraint_graph();
                let mut cache = vec![None; graph.num_nodes()];
                let (bus_index, _) = air
                    .buses
                    .iter()
                    .enumerate()
                    .find(|(_, (name, _))| name.as_str() == "bus_6_chiplets_bus")
                    .expect("missing chiplets bus");
                let s_local = ctx.aux_rows[0][bus_index];
                let s_next = ctx.aux_rows[1][bus_index];
                let coeffs = &ctx.random_values;

                let one = QuadGoldilocksOOD::ONE;
                let mut insert_groups: BTreeMap<
                    air_ir::NodeIndex,
                    (QuadGoldilocksOOD, QuadGoldilocksOOD),
                > = BTreeMap::new();
                let mut insert_group_labels: BTreeMap<air_ir::NodeIndex, Vec<u64>> =
                    BTreeMap::new();
                let mut insert_group_lens: BTreeMap<
                    air_ir::NodeIndex,
                    std::collections::BTreeSet<usize>,
                > = BTreeMap::new();
                let mut remove_groups: BTreeMap<
                    air_ir::NodeIndex,
                    (QuadGoldilocksOOD, QuadGoldilocksOOD),
                > = BTreeMap::new();

                for (op_idx, bus_op) in chiplets_bus.bus_ops.iter().enumerate() {
                    let latch_val = eval_node(graph, &bus_op.latch, &ctx, &mut cache);
                    let mut row = coeffs[0];
                    let mut col_vals = Vec::with_capacity(bus_op.columns.len());
                    for (idx, col) in bus_op.columns.iter().enumerate() {
                        let col_val = eval_node(graph, col, &ctx, &mut cache);
                        col_vals.push(col_val);
                        row = row + coeffs[idx + 1] * col_val;
                    }
                    if debug_verbose {
                        eprintln!(
                            "debug bus_6 op[{op_idx}] kind={:?} latch_node={:?} latch={}+{}X row={}+{}X cols={:?}",
                            bus_op.op_kind,
                            bus_op.latch,
                            latch_val.c0.as_int(),
                            latch_val.c1.as_int(),
                            row.c0.as_int(),
                            row.c1.as_int(),
                            col_vals
                                .iter()
                                .map(|val| format!("{}+{}X", val.c0.as_int(), val.c1.as_int()))
                                .collect::<Vec<_>>(),
                        );
                    }
                    if let air_ir::BusOpKind::Insert = bus_op.op_kind {
                        if let Some(label_val) = col_vals.first() {
                            insert_group_labels
                                .entry(bus_op.latch)
                                .or_default()
                                .push(label_val.c0.as_int());
                        }
                        insert_group_lens
                            .entry(bus_op.latch)
                            .or_default()
                            .insert(bus_op.columns.len());
                    }
                    let groups = match bus_op.op_kind {
                        air_ir::BusOpKind::Insert => &mut insert_groups,
                        air_ir::BusOpKind::Remove => &mut remove_groups,
                    };
                    groups
                        .entry(bus_op.latch)
                        .and_modify(|entry| {
                            entry.1 = entry.1 * row;
                        })
                        .or_insert((latch_val, row));
                }

                let mut insert_sum = QuadGoldilocksOOD::ZERO;
                let mut insert_flag_sum = QuadGoldilocksOOD::ZERO;
                if debug_verbose {
                    for (latch, (latch_val, msg_prod)) in &insert_groups {
                        eprintln!(
                            "debug bus_6 insert group latch={:?} latch_val={}+{}X msg_prod={}+{}X",
                            latch,
                            latch_val.c0.as_int(),
                            latch_val.c1.as_int(),
                            msg_prod.c0.as_int(),
                            msg_prod.c1.as_int(),
                        );
                    }
                }
                if debug_bus6 {
                    let mut insert_hasher = QuadGoldilocksOOD::ZERO;
                    let mut insert_bitwise = QuadGoldilocksOOD::ZERO;
                    let mut insert_memory = QuadGoldilocksOOD::ZERO;
                    let mut insert_ace = QuadGoldilocksOOD::ZERO;
                    let mut insert_kernel = QuadGoldilocksOOD::ZERO;
                    let mut insert_unknown = QuadGoldilocksOOD::ZERO;

                    let is_hasher_label =
                        |label: u64| matches!(label, 19 | 23 | 27 | 31 | 33 | 35 | 41);
                    let is_bitwise_label = |label: u64| matches!(label, 2 | 6);
                    let is_memory_label = |label: u64| matches!(label, 4 | 12 | 20 | 28);
                    let is_ace_label = |label: u64| label == 8;
                    let is_kernel_label = |label: u64| matches!(label, 16 | 48);

                    for (latch, (latch_val, msg_prod)) in &insert_groups {
                        let labels = insert_group_labels.get(latch).cloned().unwrap_or_default();
                        let lens = insert_group_lens.get(latch).cloned().unwrap_or_default();
                        let group_term = *msg_prod * *latch_val;
                        let mut classified = false;
                        if lens.len() == 1 {
                            let len = *lens.iter().next().expect("len set non-empty");
                            match len {
                                15 => {
                                    insert_hasher = insert_hasher + group_term;
                                    classified = true;
                                },
                                4 => {
                                    insert_bitwise = insert_bitwise + group_term;
                                    classified = true;
                                },
                                8 => {
                                    insert_memory = insert_memory + group_term;
                                    classified = true;
                                },
                                6 => {
                                    insert_ace = insert_ace + group_term;
                                    classified = true;
                                },
                                5 => {
                                    insert_kernel = insert_kernel + group_term;
                                    classified = true;
                                },
                                _ => {},
                            }
                        }
                        if !classified {
                            if labels.iter().any(|label| is_hasher_label(*label)) {
                                insert_hasher = insert_hasher + group_term;
                                classified = true;
                            }
                            if labels.iter().any(|label| is_bitwise_label(*label)) {
                                insert_bitwise = insert_bitwise + group_term;
                                classified = true;
                            }
                            if labels.iter().any(|label| is_memory_label(*label)) {
                                insert_memory = insert_memory + group_term;
                                classified = true;
                            }
                            if labels.iter().any(|label| is_ace_label(*label)) {
                                insert_ace = insert_ace + group_term;
                                classified = true;
                            }
                            if labels.iter().any(|label| is_kernel_label(*label)) {
                                insert_kernel = insert_kernel + group_term;
                                classified = true;
                            }
                        }
                        if !classified {
                            insert_unknown = insert_unknown + group_term;
                            eprintln!(
                                "debug bus_6 insert group latch={:?} labels={:?} lens={:?} -> unknown",
                                latch, labels, lens
                            );
                        }
                    }

                    eprintln!(
                        "debug bus_6 insert sums: hasher={}+{}X bitwise={}+{}X memory={}+{}X ace={}+{}X kernel={}+{}X unknown={}+{}X",
                        insert_hasher.c0.as_int(),
                        insert_hasher.c1.as_int(),
                        insert_bitwise.c0.as_int(),
                        insert_bitwise.c1.as_int(),
                        insert_memory.c0.as_int(),
                        insert_memory.c1.as_int(),
                        insert_ace.c0.as_int(),
                        insert_ace.c1.as_int(),
                        insert_kernel.c0.as_int(),
                        insert_kernel.c1.as_int(),
                        insert_unknown.c0.as_int(),
                        insert_unknown.c1.as_int(),
                    );
                }
                for (_latch, (latch_val, msg_prod)) in &insert_groups {
                    insert_sum = insert_sum + *msg_prod * *latch_val;
                    insert_flag_sum = insert_flag_sum + *latch_val;
                }
                let mut remove_sum = QuadGoldilocksOOD::ZERO;
                let mut remove_flag_sum = QuadGoldilocksOOD::ZERO;
                if debug_verbose {
                    for (latch, (latch_val, msg_prod)) in &remove_groups {
                        eprintln!(
                            "debug bus_6 remove group latch={:?} latch_val={}+{}X msg_prod={}+{}X",
                            latch,
                            latch_val.c0.as_int(),
                            latch_val.c1.as_int(),
                            msg_prod.c0.as_int(),
                            msg_prod.c1.as_int(),
                        );
                    }
                }
                for (_latch, (latch_val, msg_prod)) in &remove_groups {
                    remove_sum = remove_sum + *msg_prod * *latch_val;
                    remove_flag_sum = remove_flag_sum + *latch_val;
                }

                let response = insert_sum + (one - insert_flag_sum);
                let request = remove_sum + (one - remove_flag_sum);
                if debug_bus6 {
                    eprintln!(
                        "debug bus_6 groups: insert_sum={}+{}X insert_flag_sum={}+{}X remove_sum={}+{}X remove_flag_sum={}+{}X",
                        insert_sum.c0.as_int(),
                        insert_sum.c1.as_int(),
                        insert_flag_sum.c0.as_int(),
                        insert_flag_sum.c1.as_int(),
                        remove_sum.c0.as_int(),
                        remove_sum.c1.as_int(),
                        remove_flag_sum.c0.as_int(),
                        remove_flag_sum.c1.as_int(),
                    );
                }
                let lhs = s_next * request - s_local * response;
                let expected_quad = QuadGoldilocksOOD::from_quad(*expected_value);
                let expected_unscaled = expected_quad * ctx.transition.inv();
                let actual_unscaled =
                    QuadGoldilocksOOD::from_quad(*actual_value) * ctx.transition.inv();
                eprintln!(
                    "debug bus_6: s_local={}+{}X s_next={}+{}X transition={}+{}X",
                    s_local.c0.as_int(),
                    s_local.c1.as_int(),
                    s_next.c0.as_int(),
                    s_next.c1.as_int(),
                    ctx.transition.c0.as_int(),
                    ctx.transition.c1.as_int(),
                );
                eprintln!(
                    "debug bus_6: request={}+{}X response={}+{}X",
                    request.c0.as_int(),
                    request.c1.as_int(),
                    response.c0.as_int(),
                    response.c1.as_int(),
                );
                eprintln!(
                    "debug bus_6: lhs={}+{}X actual_unscaled={}+{}X expected_unscaled={}+{}X",
                    lhs.c0.as_int(),
                    lhs.c1.as_int(),
                    actual_unscaled.c0.as_int(),
                    actual_unscaled.c1.as_int(),
                    expected_unscaled.c0.as_int(),
                    expected_unscaled.c1.as_int(),
                );
                let expected_response = (s_next * request - expected_unscaled) * s_local.inv();
                let expected_request = (expected_unscaled + s_local * response) * s_next.inv();
                eprintln!(
                    "debug bus_6: expected_response_if_request_ok={}+{}X",
                    expected_response.c0.as_int(),
                    expected_response.c1.as_int(),
                );
                eprintln!(
                    "debug bus_6: expected_request_if_response_ok={}+{}X",
                    expected_request.c0.as_int(),
                    expected_request.c1.as_int(),
                );
            }
            if *actual_name == "stack.overflow.bus.transition" {
                let graph = air.constraint_graph();
                let mut cache = vec![None; graph.num_nodes()];
                let (stack_bus_index, (_, stack_bus)) = air
                    .buses
                    .iter()
                    .enumerate()
                    .find(|(_, (name, _))| name.as_str() == "bus_3_stack_p1")
                    .expect("missing stack overflow bus");
                let s_local = ctx.aux_rows[0][stack_bus_index];
                let s_next = ctx.aux_rows[1][stack_bus_index];
                let coeff0 = ctx.random_values[0];
                let coeff1 = ctx.random_values[1];
                let coeff2 = ctx.random_values[2];
                let coeff3 = ctx.random_values[3];
                let beta = coeff2;
                let beta2 = coeff3;
                let beta3 = beta2 * beta;
                eprintln!(
                    "debug coefficients: alpha={}+{}X one={}+{}X beta={}+{}X beta2={}+{}X beta3={}+{}X",
                    coeff0.c0.as_int(),
                    coeff0.c1.as_int(),
                    coeff1.c0.as_int(),
                    coeff1.c1.as_int(),
                    beta.c0.as_int(),
                    beta.c1.as_int(),
                    beta2.c0.as_int(),
                    beta2.c1.as_int(),
                    beta3.c0.as_int(),
                    beta3.c1.as_int(),
                );

                let one = QuadGoldilocksOOD::from_felt(Felt::ONE);
                let mut insert_sum = QuadGoldilocksOOD::ZERO;
                let mut insert_flag_sum = QuadGoldilocksOOD::ZERO;
                let mut remove_sum = QuadGoldilocksOOD::ZERO;
                let mut remove_flag_sum = QuadGoldilocksOOD::ZERO;
                let mut insert_sum_alt = QuadGoldilocksOOD::ZERO;
                let mut insert_flag_sum_alt = QuadGoldilocksOOD::ZERO;
                let mut remove_sum_alt = QuadGoldilocksOOD::ZERO;
                let mut remove_flag_sum_alt = QuadGoldilocksOOD::ZERO;

                // Expected raw column values from the trace layout.
                let system_offset = 0usize;
                let decoder_offset = 6usize;
                let stack_offset = decoder_offset + 24;
                let clk = QuadGoldilocksOOD::from_felt(ctx.main_rows[0][system_offset]);
                let s15 = QuadGoldilocksOOD::from_felt(ctx.main_rows[0][stack_offset + 15]);
                let s15_next = QuadGoldilocksOOD::from_felt(ctx.main_rows[1][stack_offset + 15]);
                let b1 = QuadGoldilocksOOD::from_felt(ctx.main_rows[0][stack_offset + 17]);
                let b1_next = QuadGoldilocksOOD::from_felt(ctx.main_rows[1][stack_offset + 17]);
                let h5 = QuadGoldilocksOOD::from_felt(ctx.main_rows[0][decoder_offset + 13]);
                eprintln!(
                    "debug trace inputs: clk={} s15={} s15'={} b1={} b1'={} h5={}",
                    clk.c0.as_int(),
                    s15.c0.as_int(),
                    s15_next.c0.as_int(),
                    b1.c0.as_int(),
                    b1_next.c0.as_int(),
                    h5.c0.as_int(),
                );
                eprintln!(
                    "debug s_aux: local={}+{}X next={}+{}X transition={} ",
                    s_local.c0.as_int(),
                    s_local.c1.as_int(),
                    s_next.c0.as_int(),
                    s_next.c1.as_int(),
                    ctx.transition.c0.as_int(),
                );
                for bus_op in &stack_bus.bus_ops {
                    let latch = eval_node(graph, &bus_op.latch, &ctx, &mut cache);
                    let cols: Vec<_> = bus_op
                        .columns
                        .iter()
                        .map(|col| eval_node(graph, col, &ctx, &mut cache))
                        .collect();
                    let row = coeff0 + coeff1 * cols[0] + coeff2 * cols[1] + coeff3 * cols[2];
                    let row_alt = coeff0 + beta * cols[0] + beta2 * cols[1] + beta3 * cols[2];
                    eprintln!(
                        "debug bus op {:?} latch={} cols=[{}, {}, {}]",
                        bus_op.op_kind,
                        latch.c0.as_int(),
                        cols[0].c0.as_int(),
                        cols[1].c0.as_int(),
                        cols[2].c0.as_int(),
                    );
                    match bus_op.op_kind {
                        air_ir::BusOpKind::Insert => {
                            insert_sum = insert_sum + row * latch;
                            insert_flag_sum = insert_flag_sum + latch;
                            insert_sum_alt = insert_sum_alt + row_alt * latch;
                            insert_flag_sum_alt = insert_flag_sum_alt + latch;
                        },
                        air_ir::BusOpKind::Remove => {
                            remove_sum = remove_sum + row * latch;
                            remove_flag_sum = remove_flag_sum + latch;
                            remove_sum_alt = remove_sum_alt + row_alt * latch;
                            remove_flag_sum_alt = remove_flag_sum_alt + latch;
                        },
                    }
                }
                let response = insert_sum + (one - insert_flag_sum);
                let request = remove_sum + (one - remove_flag_sum);
                let alt_expr = s_next * request - s_local * response;
                let alt_expr_t = alt_expr * ctx.transition;
                let response_alt = insert_sum_alt + (one - insert_flag_sum_alt);
                let request_alt = remove_sum_alt + (one - remove_flag_sum_alt);
                let alt_expr_alt = s_next * request_alt - s_local * response_alt;
                let alt_expr_alt_t = alt_expr_alt * ctx.transition;
                eprintln!(
                    "debug std_expr_t={}+{}X alt_expr_t={}+{}X",
                    alt_expr_t.c0.as_int(),
                    alt_expr_t.c1.as_int(),
                    alt_expr_alt_t.c0.as_int(),
                    alt_expr_alt_t.c1.as_int(),
                );
            }
        }
        // Preserve both ordering and numeric equivalence with miden-vm.
        assert_eq!(actual_value, expected_value);
    }
}

#[test]
fn test_miden_vm_minimal_ood_evals_match() {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(run_miden_vm_minimal_ood_evals_match)
        .expect("spawn OOD eval thread")
        .join()
        .expect("OOD eval thread panicked");
}
