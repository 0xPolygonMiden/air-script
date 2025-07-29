use std::collections::{BTreeMap, HashMap};

use air_parser::ast::TraceSegmentId;
use mir::ir::{QuadFelt, const_quad_felt, query_hashed_cur_eval, query_indexed_cur_eval};
use rand::prelude::*;
use winter_math::fields::f64::BaseElement as Felt;

use crate::{
    AlgebraicGraph, CompileError, NodeIndex, Operation, PeriodicColumnAccess, PublicInputAccess,
    PublicInputTableAccess, Value,
};
/// Represents the current existing evaluations to persist random values taken by the same values.
#[derive(Debug, Clone, Default)]
pub struct RandomInputs {
    rng: ThreadRng,
    // A vector to hold the random values taken for the main trace, indexed in the following way:
    // $main[0], $main[0]', $main[1], $main[1]', $main[2], ...
    main_trace: Vec<QuadFelt>,
    aux_trace: Vec<QuadFelt>,
    rand_values: Vec<QuadFelt>,
    public_inputs: HashMap<PublicInputAccess, QuadFelt>,
    periodic_columns: HashMap<PeriodicColumnAccess, QuadFelt>,
    public_inputs_tables: HashMap<PublicInputTableAccess, QuadFelt>,
    // A map to hold the the current evaluations of nodes at random points
    pub evals_map: BTreeMap<NodeIndex, QuadFelt>,
}

impl RandomInputs {
    /// Evaluates a given MIR node at random points.
    ///
    /// Note that we currently assume this will be called only during the unrolling phase, some
    /// operation types are not handled.
    pub fn eval(
        &mut self,
        graph: &AlgebraicGraph,
        node_index: &NodeIndex,
    ) -> Result<QuadFelt, CompileError> {
        let op = graph.node(node_index).op();
        match op {
            Operation::Add(lhs, rhs) => {
                let lhs_eval = self.eval(graph, lhs)?;
                let rhs_eval = self.eval(graph, rhs)?;
                let add_eval = lhs_eval + rhs_eval;
                self.evals_map.insert(*node_index, add_eval);
                Ok(add_eval)
            },
            Operation::Sub(lhs, rhs) => {
                let lhs_eval = self.eval(graph, lhs)?;
                let rhs_eval = self.eval(graph, rhs)?;
                let sub_eval = lhs_eval - rhs_eval;
                self.evals_map.insert(*node_index, sub_eval);
                Ok(sub_eval)
            },
            Operation::Mul(lhs, rhs) => {
                let lhs_eval = self.eval(graph, lhs)?;
                let rhs_eval = self.eval(graph, rhs)?;
                let mul_eval = lhs_eval * rhs_eval;
                self.evals_map.insert(*node_index, mul_eval);
                Ok(mul_eval)
            },
            Operation::Value(value) => match value {
                Value::Constant(c) => {
                    let felt = Felt::new(*c);
                    let eval = const_quad_felt(felt);
                    self.evals_map.insert(*node_index, eval);
                    Ok(eval)
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
                        let eval =
                            query_indexed_cur_eval(&mut self.rng, &mut self.main_trace, index);
                        self.evals_map.insert(*node_index, eval);
                        Ok(eval)
                    },
                    TraceSegmentId::Aux => {
                        let index = trace_access.column * 2 + trace_access.row_offset;
                        let eval =
                            query_indexed_cur_eval(&mut self.rng, &mut self.aux_trace, index);
                        self.evals_map.insert(*node_index, eval);
                        Ok(eval)
                    },
                },
                Value::RandomValue(u) => {
                    let eval = query_indexed_cur_eval(&mut self.rng, &mut self.rand_values, *u);
                    self.evals_map.insert(*node_index, eval);
                    Ok(eval)
                },
                // For PublicInput, PeriodicColumn and PublicInputTable, we use the Hash of the
                // element to associate a unique random value or each public input
                // and each periodic column access
                Value::PublicInput(pi) => {
                    let eval = query_hashed_cur_eval(&mut self.rng, &mut self.public_inputs, pi);
                    self.evals_map.insert(*node_index, eval);
                    Ok(eval)
                },
                Value::PeriodicColumn(pc) => {
                    let eval = query_hashed_cur_eval(&mut self.rng, &mut self.periodic_columns, pc);
                    self.evals_map.insert(*node_index, eval);
                    Ok(eval)
                },
                Value::PublicInputTable(public_input_table_access) => {
                    let eval = query_hashed_cur_eval(
                        &mut self.rng,
                        &mut self.public_inputs_tables,
                        public_input_table_access,
                    );
                    self.evals_map.insert(*node_index, eval);
                    Ok(eval)
                },
            },
        }
    }
}
