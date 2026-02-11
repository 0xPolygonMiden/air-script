//! OOD evaluation parity test for the minimal MidenVM AIR.
//!
//! We mirror the miden-vm evaluation flow and compare against fixed outputs captured from miden-vm.

use std::sync::Arc;

use air_ir::{
    compile, Air, AlgebraicGraph, ConstraintDomain, ConstraintRoot, NodeIndex, Operation,
    TraceSegmentId, Value,
};
use air_mir::ir::QuadFelt;
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
/// This order must match the constraint IDs (0..7).
const CONSTRAINT_NAMES: [&str; 8] = [
    "system.clk.first_row",
    "system.clk.transition",
    "range.main.v.first_row",
    "range.main.v.last_row",
    "range.main.v.transition",
    "range.bus.first_row",
    "range.bus.last_row",
    "range.bus.transition",
];

// === Helpers ===

/// Loads the minimal MidenVM AIR example from `constraints/miden_vm_minimal.air`.
pub fn load_miden_vm_minimal_air() -> std::io::Result<String> {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = format!("{}/miden_vm_minimal.air", crate_dir);
    let content = std::fs::read_to_string(path)?;

    Ok(content)
}

/// Parses and lowers AIRScript into AIR for evaluation.
fn generate_air(source: &str) -> Air {
    let code_map = Arc::new(CodeMap::new());
    let emitter = Arc::new(DefaultEmitter::new(ColorChoice::Auto));
    let diagnostics = DiagnosticsHandler::new(Default::default(), code_map.clone(), emitter);

    let air = air_parser::parse(&diagnostics, code_map, source)
        .map_err(air_ir::CompileError::Parse)
        .and_then(|program| compile(&diagnostics, program))
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
    /// Random values consumed by the AIR (alpha, then a fixed 1 for stability).
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
        assert_eq!(air.num_random_values, 2, "unexpected randomness count for minimal AIR");

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
        let alpha = randomness[0];

        // Consume aux bus boundary values to match miden-vm RNG sequence.
        for _ in 0..aux_width {
            let _ = rng.next_quad();
        }

        let first_row = QuadGoldilocksOOD::from_felt(rng.next_felt());
        let last_row = QuadGoldilocksOOD::from_felt(rng.next_felt());
        let transition = QuadGoldilocksOOD::from_felt(rng.next_felt());

        // random_values = [alpha, 1]; the second slot keeps random indexing stable.
        let random_values = vec![alpha, QuadGoldilocksOOD::from_felt(Felt::new(1))];

        Self {
            main_rows,
            aux_rows,
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
        .map(|(_, constraint)| eval_constraint(graph, constraint, ctx, &mut cache))
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
    let value = eval_node(graph, root.node_index(), ctx, cache);
    match root.domain() {
        ConstraintDomain::FirstRow => value * ctx.first_row,
        ConstraintDomain::LastRow => value * ctx.last_row,
        ConstraintDomain::EveryFrame(2) => value * ctx.transition,
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
    let idx: usize = (*index).into();
    if let Some(value) = cache[idx] {
        return value;
    }

    let value = match graph.node(index).op() {
        Operation::Value(Value::Constant(value)) => QuadGoldilocksOOD::from_felt(Felt::new(*value)),
        Operation::Value(Value::RandomValue(index)) => {
            ctx.random_values.get(*index).copied().expect("random index out of bounds")
        },
        Operation::Value(Value::TraceAccess(access)) => {
            let row = access.row_offset;
            match access.segment {
                TraceSegmentId::Main => {
                    let value = ctx
                        .main_rows
                        .get(row)
                        .and_then(|row| row.get(access.column))
                        .copied()
                        .expect("main trace access out of bounds");
                    QuadGoldilocksOOD::from_felt(value)
                },
                TraceSegmentId::Aux => ctx
                    .aux_rows
                    .get(row)
                    .and_then(|row| row.get(access.column))
                    .copied()
                    .expect("aux trace access out of bounds"),
            }
        },
        Operation::Value(Value::PeriodicColumn(_)) => {
            panic!("periodic columns are not expected in minimal OOD eval");
        },
        Operation::Value(Value::PublicInput(_)) => {
            panic!("public inputs are not expected in minimal OOD eval");
        },
        Operation::Value(Value::PublicInputTable(_)) => {
            panic!("public input tables are not expected in minimal OOD eval");
        },
        Operation::Add(lhs, rhs) => {
            eval_node(graph, lhs, ctx, cache) + eval_node(graph, rhs, ctx, cache)
        },
        Operation::Sub(lhs, rhs) => {
            eval_node(graph, lhs, ctx, cache) - eval_node(graph, rhs, ctx, cache)
        },
        Operation::Mul(lhs, rhs) => {
            eval_node(graph, lhs, ctx, cache) * eval_node(graph, rhs, ctx, cache)
        },
    };

    cache[idx] = Some(value);
    value
}

// === OOD expected outputs ===

fn expected_ood_evals() -> Vec<(&'static str, QuadFelt)> {
    // These values are sourced from the miden-vm OOD evaluator.
    // Update them only by re-running the miden-vm OOD dump test.
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
            "range.bus.first_row",
            QuadFelt::new(Felt::new(12608813705579209032), Felt::new(3989096837606726344)),
        ),
        (
            "range.bus.last_row",
            QuadFelt::new(Felt::new(377034121616931435), Felt::new(3703916915744149174)),
        ),
        (
            "range.bus.transition",
            QuadFelt::new(Felt::new(10365289165200035540), Felt::new(16469718665506609592)),
        ),
    ]
}

/// Computes OOD evaluations in tag order using Goldilocks extension arithmetic.
fn ood_evals_miden_vm_minimal() -> Vec<(&'static str, QuadFelt)> {
    let air_string = load_miden_vm_minimal_air().expect("unable to read MidenVM_Minimal AIR");
    let air = generate_air(&air_string);

    let ctx = OodContext::new(&air, OOD_SEED);
    let evals = eval_constraints_in_id_order(&air, &ctx);

    assert_eq!(evals.len(), CONSTRAINT_NAMES.len());
    CONSTRAINT_NAMES
        .iter()
        .copied()
        .zip(evals.into_iter().map(|value| value.to_quad()))
        .collect()
}

// === Tests ===

#[test]
fn test_miden_vm_minimal_ood_evals_match() {
    let expected = expected_ood_evals();
    let actual = ood_evals_miden_vm_minimal();
    assert_eq!(actual.len(), expected.len());
    for ((actual_name, actual_value), (expected_name, expected_value)) in
        actual.iter().zip(expected.iter())
    {
        assert_eq!(actual_name, expected_name);
        // Preserve both ordering and numeric equivalence with miden-vm.
        assert_eq!(actual_value, expected_value);
    }

    // Print tag ordering and evaluations to aid manual inspection.
    let air_string = load_miden_vm_minimal_air().expect("unable to read MidenVM_Minimal AIR");
    let air = generate_air(&air_string);
    let tagged = collect_tagged_constraints(&air);
    let ctx = OodContext::new(&air, OOD_SEED);
    let evals = eval_constraints_in_id_order(&air, &ctx);

    println!("tagged constraints (tag -> segment/domain -> name -> eval):");
    for (((tag, segment, domain), name), value) in
        tagged.iter().zip(CONSTRAINT_NAMES.iter()).zip(evals.iter())
    {
        let quad = value.to_quad();
        let [c0, c1] = quad.to_base_elements();
        println!("  {tag}: {segment:?}/{domain:?} {name} = {} + {} X", c0.as_int(), c1.as_int());
    }
}
