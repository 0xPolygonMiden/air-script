use std::{
    collections::{BTreeMap, HashMap},
    hash::{Hash, Hasher},
    ops::{Add, Mul, Sub},
};

use rand::{distr::Uniform, prelude::*};
use winter_math::{StarkField, fields::f64::BaseElement as Felt};

use crate::{
    AlgebraicGraph, CompileError, NodeIndex, Operation, PeriodicColumnAccess, PublicInputAccess,
    PublicInputTableAccess, Value,
};

pub const NUM_EVALS: usize = 3; // Number of random evaluation points

/// A struct representing the evaluation of an operation at `NUM_EVALS` random points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eval<const NUM_EVALS: usize>(pub [Felt; NUM_EVALS]);

impl<const NUM_EVALS: usize> Eval<NUM_EVALS> {
    /// Creates a new [Eval] with random values.
    fn new_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        // Note: using a uniform distribution over all u64 values would lead to a non-uniform
        // distribution of Felt values.
        let distr = Uniform::new(0, Felt::MODULUS).unwrap(); // Unwrap is safe as Felt::MODULUS is > 0
        Eval(
            (0..NUM_EVALS)
                .map(|_| Felt::new(rng.sample(distr)))
                .collect::<Vec<Felt>>()
                .try_into()
                .unwrap(),
        )
    }

    /// Creates a new [Eval] with the same constant value for all evaluations.
    fn new_const(felt: Felt) -> Self {
        Eval([felt; NUM_EVALS])
    }
}

impl<const NUM_EVALS: usize> Add for Eval<NUM_EVALS> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Eval(
            self.0
                .iter()
                .zip(other.0.iter())
                .map(|(a, b)| *a + *b)
                .collect::<Vec<Felt>>()
                .try_into()
                .unwrap(),
        )
    }
}

impl<const NUM_EVALS: usize> Sub for Eval<NUM_EVALS> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Eval(
            self.0
                .iter()
                .zip(other.0.iter())
                .map(|(a, b)| *a - *b)
                .collect::<Vec<Felt>>()
                .try_into()
                .unwrap(),
        )
    }
}

impl<const NUM_EVALS: usize> Mul for Eval<NUM_EVALS> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Eval(
            self.0
                .iter()
                .zip(other.0.iter())
                .map(|(a, b)| *a * *b)
                .collect::<Vec<Felt>>()
                .try_into()
                .unwrap(),
        )
    }
}

impl<const NUM_EVALS: usize> Hash for Eval<NUM_EVALS> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for felt in &self.0 {
            felt.as_int().hash(state);
        }
    }
}

impl<const NUM_EVALS: usize> Ord for Eval<NUM_EVALS> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .iter()
            .zip(other.0.iter())
            .fold(std::cmp::Ordering::Equal, |acc, (a, b)| {
                if acc != std::cmp::Ordering::Equal {
                    acc
                } else {
                    a.as_int().cmp(&b.as_int())
                }
            })
    }
}

impl<const NUM_EVALS: usize> PartialOrd for Eval<NUM_EVALS> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents the current existing evaluations to persist random values taken by the same values.
#[derive(Debug, Clone, Default)]
pub struct CurrentEvals<const NUM_EVALS: usize> {
    main_trace: Vec<Eval<NUM_EVALS>>,
    aux_trace: Vec<Eval<NUM_EVALS>>,
    rand_values: Vec<Eval<NUM_EVALS>>,
    public_inputs: HashMap<PublicInputAccess, Eval<NUM_EVALS>>,
    periodic_columns: HashMap<PeriodicColumnAccess, Eval<NUM_EVALS>>,
    public_inputs_tables: HashMap<PublicInputTableAccess, Eval<NUM_EVALS>>,
}

/// Helper function to either query an existing evaluation or create a new random one if the index
/// is out of bounds.
fn query_cur_eval<R: Rng + ?Sized>(
    rng: &mut R,
    cur_eval_vec: &mut Vec<Eval<NUM_EVALS>>,
    index: usize,
) -> Eval<NUM_EVALS> {
    if cur_eval_vec.len() <= index {
        cur_eval_vec.resize_with(index + 1, || Eval::new_random(rng));
    }
    cur_eval_vec[index]
}

/// Helper function to either query an existing evaluation or create a new random one if the element
/// is not present in the map.
fn query_hashed_cur_eval<R: Rng + ?Sized, H: Hash + Eq + Clone>(
    rng: &mut R,
    cur_eval_map: &mut HashMap<H, Eval<NUM_EVALS>>,
    element: &H,
) -> Eval<NUM_EVALS> {
    if cur_eval_map.get(element).is_none() {
        let eval = Eval::new_random(rng);
        cur_eval_map.insert(element.clone(), eval);
    }
    *cur_eval_map.get(element).unwrap()
}

/// Evaluates a given MIR node at random points.
pub fn eval_random_point<R: Rng + ?Sized>(
    rng: &mut R,
    current_evals: &mut CurrentEvals<NUM_EVALS>,
    evals_map: &mut BTreeMap<NodeIndex, Eval<NUM_EVALS>>,
    graph: &AlgebraicGraph,
    node_index: &NodeIndex,
) -> Result<Eval<NUM_EVALS>, CompileError> {
    let op = graph.node(node_index).op();
    match op {
        Operation::Add(lhs, rhs) => {
            let lhs_eval = eval_random_point(rng, current_evals, evals_map, graph, lhs)?;
            let rhs_eval = eval_random_point(rng, current_evals, evals_map, graph, rhs)?;
            let add_eval = lhs_eval + rhs_eval;
            evals_map.insert(*node_index, add_eval);
            Ok(add_eval)
        },
        Operation::Sub(lhs, rhs) => {
            let lhs_eval = eval_random_point(rng, current_evals, evals_map, graph, lhs)?;
            let rhs_eval = eval_random_point(rng, current_evals, evals_map, graph, rhs)?;
            let sub_eval = lhs_eval - rhs_eval;
            evals_map.insert(*node_index, sub_eval);
            Ok(sub_eval)
        },
        Operation::Mul(lhs, rhs) => {
            let lhs_eval = eval_random_point(rng, current_evals, evals_map, graph, lhs)?;
            let rhs_eval = eval_random_point(rng, current_evals, evals_map, graph, rhs)?;
            let mul_eval = lhs_eval * rhs_eval;
            evals_map.insert(*node_index, mul_eval);
            Ok(mul_eval)
        },
        Operation::Value(value) => match value {
            Value::Constant(c) => {
                let felt = Felt::new(*c);
                let eval = Eval::new_const(felt);
                evals_map.insert(*node_index, eval);
                Ok(eval)
            },
            Value::PublicInput(pi) => {
                let eval = query_hashed_cur_eval(rng, &mut current_evals.public_inputs, pi);
                evals_map.insert(*node_index, eval);
                Ok(eval)
            },
            Value::RandomValue(u) => {
                let eval = query_cur_eval(rng, &mut current_evals.rand_values, *u);
                evals_map.insert(*node_index, eval);
                Ok(eval)
            },
            Value::TraceAccess(trace_access) => match trace_access.segment {
                0 => {
                    let index = trace_access.column * 2 + trace_access.row_offset;
                    let eval = query_cur_eval(rng, &mut current_evals.main_trace, index);
                    evals_map.insert(*node_index, eval);
                    Ok(eval)
                },
                1 => {
                    let index = trace_access.column * 2 + trace_access.row_offset;
                    let eval = query_cur_eval(rng, &mut current_evals.aux_trace, index);
                    evals_map.insert(*node_index, eval);
                    Ok(eval)
                },
                _ => {
                    println!("Unexpected segment in eval_random_point: {}", trace_access.segment);
                    Err(CompileError::Failed)
                },
            },
            Value::PeriodicColumn(pc) => {
                let eval = query_hashed_cur_eval(rng, &mut current_evals.periodic_columns, pc);
                evals_map.insert(*node_index, eval);
                Ok(eval)
            },
            Value::PublicInputTable(public_input_table_access) => {
                let eval = query_hashed_cur_eval(
                    rng,
                    &mut current_evals.public_inputs_tables,
                    public_input_table_access,
                );
                evals_map.insert(*node_index, eval);
                Ok(eval)
            },
        },
    }
}
