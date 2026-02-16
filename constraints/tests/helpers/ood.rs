use std::{collections::BTreeMap, sync::Arc};

use air_ir::{
    passes::{BusOpExpand, CommonSubexpressionElimination, MirToAir, TagValidation},
    Air, AlgebraicGraph, ConstraintDomain, ConstraintRoot, NodeIndex, Operation, TraceSegmentId,
    Value,
};
use air_mir::{ir::QuadFelt, MirPasses};
use air_parser::AstPasses;
use air_pass::Pass;
use miden_core::{Felt, FieldElement};
use miden_diagnostics::{
    term::termcolor::ColorChoice, CodeMap, DefaultEmitter, DiagnosticsHandler,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use winter_utils::Randomizable;

use crate::fixtures::ood_expected::EvalRecord;

/// OOD evaluation configuration.
#[derive(Clone, Copy)]
pub struct OodConfig {
    pub seed: u64,
    pub aux_trace_rand_elements: usize,
    pub expected_main_width: usize,
    pub expected_aux_width: usize,
    pub main_filled: usize,
    pub ungated_transition_tags: &'static [usize],
    pub periodic_column_order: &'static [&'static str],
}

/// Inputs needed to run an OOD parity test.
pub struct OodTest<'a> {
    pub air_path: &'a str,
    pub config: OodConfig,
    pub expected: Vec<EvalRecord>,
    pub constraint_names: &'static [&'static str],
}

/// Manifest describing a tagged constraint group.
pub struct GroupManifest<'a> {
    pub air_path: &'a str,
    pub expected: Vec<EvalRecord>,
    pub constraint_names: &'static [&'static str],
}

impl<'a> GroupManifest<'a> {
    pub fn new(
        air_path: &'a str,
        expected: Vec<EvalRecord>,
        constraint_names: &'static [&'static str],
    ) -> Self {
        Self { air_path, expected, constraint_names }
    }
}

const DEFAULT_OOD_SEED: u64 = 0xC0FFEE;
const DEFAULT_AUX_TRACE_RAND_ELEMENTS: usize = 16;
const DEFAULT_EXPECTED_MAIN_WIDTH: usize = 72;
const DEFAULT_EXPECTED_AUX_WIDTH: usize = 8;
const DEFAULT_MAIN_FILLED: usize = 71;
const DEFAULT_UNGATED_TRANSITION_TAGS: &'static [usize] = &[];
const DEFAULT_PERIODIC_COLUMN_ORDER: &'static [&'static str] = &[];

/// Parses and lowers AIRScript into AIR for evaluation.
pub fn generate_air(path: &str) -> Air {
    if std::env::var("AIR_ENABLE_CSE").is_ok() {
        generate_air_with_cse(path)
    } else {
        generate_air_without_cse(path)
    }
}

/// Runs an OOD parity test and prints mismatches with tag/domain context.
pub fn run_ood_parity_test(test: OodTest<'_>) {
    let air = generate_air(test.air_path);
    let tagged = collect_tagged_constraints(&air);
    let ctx = OodContext::new(&air, &test.config);
    let actual_values = eval_constraints_in_id_order(
        &air,
        &ctx,
        test.expected.len(),
        test.config.ungated_transition_tags,
    );

    assert_eq!(actual_values.len(), test.expected.len());
    let actual: Vec<EvalRecord> = test
        .constraint_names
        .iter()
        .enumerate()
        .map(|(id, name)| EvalRecord {
            id,
            name,
            value: actual_values[id].to_quad(),
        })
        .collect();

    assert_eq!(actual.len(), test.expected.len());
    for (idx, (actual, expected)) in actual.iter().zip(test.expected.iter()).enumerate() {
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.name, expected.name);
        if actual.value != expected.value {
            let [a0, a1] = actual.value.to_base_elements();
            let [e0, e1] = expected.value.to_base_elements();
            let (tag, segment, domain) = tagged[idx];
            eprintln!(
                "mismatch at #{idx} tag={tag} {segment:?}/{domain:?} {}: actual={}+{}X expected={}+{}X",
                actual.name,
                a0.as_int(),
                a1.as_int(),
                e0.as_int(),
                e1.as_int()
            );
        }
        assert_eq!(actual.value, expected.value);
    }
}

/// Runs an OOD parity test for a group manifest.
pub fn run_group_parity_test(manifest: GroupManifest<'_>) {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let air_path = format!("{}/{}", crate_dir, manifest.air_path);
    let config = default_config();
    let test = OodTest {
        air_path: &air_path,
        config,
        expected: manifest.expected,
        constraint_names: manifest.constraint_names,
    };
    run_ood_parity_test(test);
}

fn default_config() -> OodConfig {
    OodConfig {
        seed: DEFAULT_OOD_SEED,
        aux_trace_rand_elements: DEFAULT_AUX_TRACE_RAND_ELEMENTS,
        expected_main_width: DEFAULT_EXPECTED_MAIN_WIDTH,
        expected_aux_width: DEFAULT_EXPECTED_AUX_WIDTH,
        main_filled: DEFAULT_MAIN_FILLED,
        ungated_transition_tags: DEFAULT_UNGATED_TRANSITION_TAGS,
        periodic_column_order: DEFAULT_PERIODIC_COLUMN_ORDER,
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

/// OOD eval uses Goldilocks quadratic extension (x^2 - 7) to match Plonky3/Miden VM.
/// This is a test-only workaround until we depend on Plonky3 types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuadGoldilocksOOD {
    c0: Felt,
    c1: Felt,
}

impl QuadGoldilocksOOD {
    pub const ZERO: Self = Self { c0: Felt::ZERO, c1: Felt::ZERO };
    #[allow(dead_code)]
    pub const ONE: Self = Self { c0: Felt::ONE, c1: Felt::ZERO };
    pub fn from_felt(value: Felt) -> Self {
        Self { c0: value, c1: Felt::ZERO }
    }

    pub fn from_quad(value: QuadFelt) -> Self {
        let [c0, c1] = value.to_base_elements();
        Self { c0, c1 }
    }

    pub fn to_quad(self) -> QuadFelt {
        QuadFelt::new(self.c0, self.c1)
    }

    #[allow(dead_code)]
    pub fn inv(self) -> Self {
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
pub struct OodContext {
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
    pub fn new(air: &Air, config: &OodConfig) -> Self {
        let main_width = air.trace_segment_widths[usize::from(TraceSegmentId::Main)] as usize;
        let aux_width = air.trace_segment_widths[usize::from(TraceSegmentId::Aux)] as usize;
        assert_eq!(
            main_width, config.expected_main_width,
            "unexpected main trace width for OOD AIR"
        );
        assert_eq!(aux_width, config.expected_aux_width, "unexpected aux trace width for OOD AIR");
        assert!(
            usize::from(air.num_random_values) <= config.aux_trace_rand_elements,
            "unexpected randomness count for OOD AIR"
        );
        assert!(config.main_filled <= main_width, "main_filled exceeds main trace width");

        let mut rng = SeededRng::new(config.seed);

        let mut main_rows = [vec![Felt::new(0); main_width], vec![Felt::new(0); main_width]];
        for row in 0..2 {
            for col in 0..config.main_filled {
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

        let mut randomness = Vec::with_capacity(config.aux_trace_rand_elements);
        for _ in 0..config.aux_trace_rand_elements {
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
            config.periodic_column_order.len(),
            "unexpected periodic column count for OOD AIR"
        );
        let mut periodic_values = BTreeMap::new();
        for name in config.periodic_column_order {
            periodic_values.insert(*name, rng.next_felt());
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

/// Evaluates constraints in tag order to match constraint IDs.
pub fn eval_constraints_in_id_order(
    air: &Air,
    ctx: &OodContext,
    expected_len: usize,
    ungated_transition_tags: &[usize],
) -> Vec<QuadGoldilocksOOD> {
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
    assert_eq!(tagged.len(), expected_len);
    for (expected, (tag, _)) in tagged.iter().enumerate() {
        assert_eq!(*tag as usize, expected, "unexpected tag ordering");
    }

    let graph = air.constraint_graph();
    let mut cache = vec![None; graph.num_nodes()];
    tagged
        .into_iter()
        .map(|(tag, constraint)| {
            let value = eval_constraint(graph, constraint, ctx, &mut cache);
            apply_domain_gate(
                value,
                constraint.domain(),
                tag as usize,
                ctx,
                ungated_transition_tags,
            )
        })
        .collect()
}

/// Collects constraint tags with their segment/domain, sorted by tag.
pub fn collect_tagged_constraints(air: &Air) -> Vec<(u64, TraceSegmentId, ConstraintDomain)> {
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
    ungated_transition_tags: &[usize],
) -> QuadGoldilocksOOD {
    match domain {
        ConstraintDomain::FirstRow => value * ctx.first_row,
        ConstraintDomain::LastRow => value * ctx.last_row,
        ConstraintDomain::EveryFrame(2) => {
            if ungated_transition_tags.contains(&tag) {
                value
            } else {
                value * ctx.transition
            }
        },
        ConstraintDomain::EveryRow => value,
        ConstraintDomain::EveryFrame(size) => {
            panic!("unsupported transition size {size} for OOD eval");
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
                panic!("public inputs are not expected in OOD eval");
            },
            Operation::Value(Value::PublicInputTable(_)) => {
                panic!("public input tables are not expected in OOD eval");
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
