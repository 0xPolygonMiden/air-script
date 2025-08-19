extern crate alloc;
use alloc::collections::BTreeMap;

use air_parser::ast::TraceSegmentId;
use mir::ir::{QuadFelt, const_quad_felt, query_indexed_eval, query_mapped_eval};
use rand::prelude::*;
use winter_math::fields::f64::BaseElement as Felt;

use crate::{
    AlgebraicGraph, NodeIndex, Operation, PeriodicColumnAccess, PublicInputAccess,
    PublicInputTableAccess, Value,
};

/// Holds both:
/// - the random inputs taken by leaf nodes, in order to persist them across different node
///   evaluations.
/// - the evaluations of all the nodes in the graph
#[derive(Debug, Clone, Default)]
pub struct RandomInputs {
    rng: ThreadRng,
    // A vector to hold the random values taken for the main trace, indexed in the following way:
    // $main[0], $main[0]', $main[1], $main[1]', $main[2], ...
    main_trace: Vec<QuadFelt>,
    aux_trace: Vec<QuadFelt>,
    rand_values: Vec<QuadFelt>,
    public_inputs: BTreeMap<PublicInputAccess, QuadFelt>,
    periodic_columns: BTreeMap<PeriodicColumnAccess, QuadFelt>,
    public_inputs_tables: BTreeMap<PublicInputTableAccess, QuadFelt>,
    // A map to hold the the current evaluations of nodes at random points
    evals_map: BTreeMap<NodeIndex, QuadFelt>,
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
                // For each trace segment, we associate a random value to each trace access,
                // indexed in the following way, each column having two
                // distinct evaluations to account for the two possible row offsets:
                // $main[0], $main[0]', $main[1], $main[1]', $main[2], ...
                // Note: if we encounter a trace access corresponding to an index we have not
                // yet evaluated, we will randomly generate values for
                // this trace access, but also for all previous indices.
                Value::TraceAccess(trace_access) => match trace_access.segment {
                    TraceSegmentId::Main => {
                        let index = trace_access.column * 2 + trace_access.row_offset;
                        let eval = query_indexed_eval(&mut self.rng, &mut self.main_trace, index);
                        self.evals_map.insert(*node_index, eval);
                        eval
                    },
                    TraceSegmentId::Aux => {
                        let index = trace_access.column * 2 + trace_access.row_offset;
                        let eval = query_indexed_eval(&mut self.rng, &mut self.aux_trace, index);
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
