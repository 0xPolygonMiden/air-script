extern crate alloc;
use alloc::collections::BTreeMap;

use air_parser::ast::{TraceColumnIndex, TraceSegmentId};
use mir::ir::{QuadFelt, const_quad_felt, query_indexed_eval, query_mapped_eval};
use rand::{SeedableRng, rngs::StdRng};
use winter_math::fields::f64::BaseElement as Felt;

use crate::{
    AlgebraicGraph, NodeIndex, Operation, PeriodicColumnAccess, PublicInputAccess,
    PublicInputTableAccess, Value,
};

/// Holds both:
/// - the random inputs taken by leaf nodes, in order to persist them across different node
///   evaluations.
/// - the evaluations of all the nodes in the graph
#[derive(Debug, Clone)]
pub struct RandomInputs {
    rng: StdRng,
    // Maps a (column, row_offset) pair to the random value assigned to that trace cell.
    //
    // NOTE: this used to be a `Vec<QuadFelt>` indexed by `column * 2 + row_offset`, which
    // implicitly assumed `row_offset` was always 0 or 1. That assumption does not hold in
    // general: `row_offset` can be 2 or greater for constraints spanning larger frames (see
    // `ConstraintDomain::EveryFrame`), and in that case the flat index collided between
    // unrelated columns (e.g. column 0 at offset 2 and column 1 at offset 0 both mapped to
    // index 2), causing CSE to incorrectly treat two distinct trace cells as equal.
    main_trace: BTreeMap<(TraceColumnIndex, usize), QuadFelt>,
    aux_trace: BTreeMap<(TraceColumnIndex, usize), QuadFelt>,
    rand_values: Vec<QuadFelt>,
    public_inputs: BTreeMap<PublicInputAccess, QuadFelt>,
    periodic_columns: BTreeMap<PeriodicColumnAccess, QuadFelt>,
    public_inputs_tables: BTreeMap<PublicInputTableAccess, QuadFelt>,
    // A map to hold the the current evaluations of nodes at random points
    evals_map: BTreeMap<NodeIndex, QuadFelt>,
}

// Deterministic seed so CSE behavior is reproducible across runs.
const CSE_RNG_SEED: [u8; 32] = *b"AIR_SCRIPT_CSE_SEED_000000000000";

impl Default for RandomInputs {
    fn default() -> Self {
        Self {
            rng: StdRng::from_seed(CSE_RNG_SEED),
            main_trace: BTreeMap::new(),
            aux_trace: BTreeMap::new(),
            rand_values: Vec::new(),
            public_inputs: BTreeMap::new(),
            periodic_columns: BTreeMap::new(),
            public_inputs_tables: BTreeMap::new(),
            evals_map: BTreeMap::new(),
        }
    }
}

impl RandomInputs {
    /// Evaluates a given algebraic graph node at random points.
    pub fn eval(&mut self, graph: &AlgebraicGraph, node_index: &NodeIndex) -> QuadFelt {
        let op = graph.node(node_index).op();
        match op {
            Operation::Add(lhs, rhs) => {
                let lhs_eval = self.eval(graph, lhs);
                let rhs_eval = self.eval(graph, rhs);
                let add_eval = lhs_eval + rhs_eval;
                self.evals_map.insert(*node_index, add_eval);
                add_eval
            },
            Operation::Sub(lhs, rhs) => {
                let lhs_eval = self.eval(graph, lhs);
                let rhs_eval = self.eval(graph, rhs);
                let sub_eval = lhs_eval - rhs_eval;
                self.evals_map.insert(*node_index, sub_eval);
                sub_eval
            },
            Operation::Mul(lhs, rhs) => {
                let lhs_eval = self.eval(graph, lhs);
                let rhs_eval = self.eval(graph, rhs);
                let mul_eval = lhs_eval * rhs_eval;
                self.evals_map.insert(*node_index, mul_eval);
                mul_eval
            },
            Operation::Value(value) => match value {
                Value::Constant(c) => {
                    let felt = Felt::new(*c);
                    let eval = const_quad_felt(felt);
                    self.evals_map.insert(*node_index, eval);
                    eval
                },
                // For each trace segment, we associate a random value to each distinct
                // (column, row_offset) pair. Row offsets are not limited to 0/1: constraints
                // over larger frames (see `ConstraintDomain::EveryFrame`) can reference a
                // column at an arbitrary offset, so we key directly on (column, row_offset)
                // rather than deriving a flat index, which previously collided between
                // unrelated columns whenever row_offset >= 2 (e.g. column 0 at offset 2 and
                // column 1 at offset 0 both mapped to the same flat index).
                Value::TraceAccess(trace_access) => match trace_access.segment {
                    TraceSegmentId::Main => {
                        let key = (trace_access.column, trace_access.row_offset);
                        let eval = query_mapped_eval(&mut self.rng, &mut self.main_trace, &key);
                        self.evals_map.insert(*node_index, eval);
                        eval
                    },
                    TraceSegmentId::Aux => {
                        let key = (trace_access.column, trace_access.row_offset);
                        let eval = query_mapped_eval(&mut self.rng, &mut self.aux_trace, &key);
                        self.evals_map.insert(*node_index, eval);
                        eval
                    },
                },
                Value::RandomValue(u) => {
                    let eval = query_indexed_eval(&mut self.rng, &mut self.rand_values, *u);
                    self.evals_map.insert(*node_index, eval);
                    eval
                },
                // For PublicInput, PeriodicColumn and PublicInputTable, we use the Hash of the
                // element to associate a unique random value or each public input
                // and each periodic column access
                Value::PublicInput(pi) => {
                    let eval = query_mapped_eval(&mut self.rng, &mut self.public_inputs, pi);
                    self.evals_map.insert(*node_index, eval);
                    eval
                },
                Value::PeriodicColumn(pc) => {
                    let eval = query_mapped_eval(&mut self.rng, &mut self.periodic_columns, pc);
                    self.evals_map.insert(*node_index, eval);
                    eval
                },
                Value::PublicInputTable(public_input_table_access) => {
                    let eval = query_mapped_eval(
                        &mut self.rng,
                        &mut self.public_inputs_tables,
                        public_input_table_access,
                    );
                    self.evals_map.insert(*node_index, eval);
                    eval
                },
            },
        }
    }

    /// Consumes self and returns all the evaluations, ordered by `NodeIndex`.
    pub fn into_evaluations(self) -> Vec<QuadFelt> {
        self.evals_map.into_values().collect()
    }
}
