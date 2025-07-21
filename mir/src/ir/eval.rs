use std::{
    array,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    ops::{Add, Deref, Index, Mul, Sub},
};

use rand::{distr::Uniform, prelude::*};
use winter_math::{FieldElement, StarkField, fields::f64::BaseElement as Felt};

use crate::{
    CompileError,
    ir::{
        ConstantValue, Link, MirValue, Op, PeriodicColumnAccess, PublicInputAccess,
        PublicInputTableAccess,
    },
};

pub const NUM_EVALS: usize = 3; // Number of random evaluation points

/// A struct representing the evaluation of an operation at `NUM_EVALS` random points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eval<const NUM_EVALS: usize>(pub [Felt; NUM_EVALS]);

impl<const NUM_EVALS: usize> Index<usize> for Eval<NUM_EVALS> {
    type Output = Felt;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<const NUM_EVALS: usize> Eval<NUM_EVALS> {
    fn exp(self, other: Self) -> Self {
        Eval(array::from_fn(|i| self[i].exp(other[i].as_int())))
    }

    /// Creates a new [Eval] with random values.
    fn new_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        // Note: using a uniform distribution over all u64 values would lead to a non-uniform
        // distribution of Felt values.
        let distr = Uniform::new(0, Felt::MODULUS).unwrap(); // Unwrap is safe as Felt::MODULUS is > 0

        Eval(array::from_fn(|_| Felt::new(rng.sample(distr))))
    }

    /// Creates a new [Eval] with the same constant value for all evaluations.
    fn new_const(felt: Felt) -> Self {
        Eval([felt; NUM_EVALS])
    }

    /// Returns the integer representation of the evaluations.
    fn as_ints(&self) -> [u64; NUM_EVALS] {
        self.0.each_ref().map(Felt::as_int)
    }
}

impl<const NUM_EVALS: usize> Add for Eval<NUM_EVALS> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Eval(array::from_fn(|i| self[i] + other[i]))
    }
}

impl<const NUM_EVALS: usize> Sub for Eval<NUM_EVALS> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Eval(array::from_fn(|i| self[i] - other[i]))
    }
}

impl<const NUM_EVALS: usize> Mul for Eval<NUM_EVALS> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Eval(array::from_fn(|i| self[i] * other[i]))
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
        // Here we can reuse the existing comparison implementation for arrays
        self.as_ints().cmp(&other.as_ints())
    }
}

impl<const NUM_EVALS: usize> PartialOrd for Eval<NUM_EVALS> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents the current existing evaluations to persist random values taken by the same values.
#[derive(Debug, Clone, Default)]
pub struct RandomInputs<const NUM_EVALS: usize> {
    rng: ThreadRng,
    main_trace: Vec<Eval<NUM_EVALS>>,
    aux_trace: Vec<Eval<NUM_EVALS>>,
    rand_values: Vec<Eval<NUM_EVALS>>,
    public_inputs: HashMap<PublicInputAccess, Eval<NUM_EVALS>>,
    periodic_columns: HashMap<PeriodicColumnAccess, Eval<NUM_EVALS>>,
    public_input_tables: HashMap<PublicInputTableAccess, Eval<NUM_EVALS>>,
}

/// Helper function to either query an existing evaluation or create a new random one if the index
/// is out of bounds.
fn query_indexed_cur_eval<R: Rng + ?Sized>(
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
    *cur_eval_map.entry(element.clone()).or_insert_with(|| Eval::new_random(rng))
}

/// Evaluates a given MIR node at random points.
pub fn eval_random_point(
    current_evals: &mut RandomInputs<NUM_EVALS>,
    op: Link<Op>,
) -> Result<Eval<NUM_EVALS>, CompileError> {
    match op.borrow().deref() {
        Op::Enf(e) => {
            let expr = eval_random_point(current_evals, e.expr.clone())?;
            Ok(expr)
        },
        Op::Add(a) => {
            let lhs = eval_random_point(current_evals, a.lhs.clone())?;
            let rhs = eval_random_point(current_evals, a.rhs.clone())?;
            Ok(lhs + rhs)
        },
        Op::Sub(s) => {
            let lhs = eval_random_point(current_evals, s.lhs.clone())?;
            let rhs = eval_random_point(current_evals, s.rhs.clone())?;
            Ok(lhs - rhs)
        },
        Op::Mul(m) => {
            let lhs = eval_random_point(current_evals, m.lhs.clone())?;
            let rhs = eval_random_point(current_evals, m.rhs.clone())?;
            Ok(lhs * rhs)
        },
        Op::Exp(e) => {
            let lhs = eval_random_point(current_evals, e.lhs.clone())?;
            let rhs = eval_random_point(current_evals, e.rhs.clone())?;
            Ok(lhs.exp(rhs))
        },
        Op::BusOp(b) => {
            let mut hasher = DefaultHasher::new();
            "BusOp".hash(&mut hasher);
            b.hash(&mut hasher);
            let hash = hasher.finish();
            let felt = Felt::new(hash);
            Ok(Eval::new_const(felt))
        },
        Op::Parameter(_) => {
            // We cannot easily detect that two parameters refer to the same For, so we consider
            // them to be all different
            Ok(Eval::new_random(&mut current_evals.rng))
        },
        Op::Value(v) => {
            match &v.value.value {
                MirValue::Constant(ConstantValue::Felt(c)) => {
                    let felt = Felt::new(*c);
                    Ok(Eval::new_const(felt))
                },
                MirValue::TraceAccess(trace_access) => match trace_access.segment {
                    0 => {
                        let index = trace_access.column * 2 + trace_access.row_offset;
                        Ok(query_indexed_cur_eval(&mut current_evals.rng, &mut current_evals.main_trace, index))
                    },
                    1 => {
                        let index = trace_access.column * 2 + trace_access.row_offset;
                        Ok(query_indexed_cur_eval(&mut current_evals.rng, &mut current_evals.aux_trace, index))
                    },
                    _ => {
                        println!(
                            "Unexpected segment in eval_random_point: {}",
                            trace_access.segment
                        );
                        Err(CompileError::Failed)
                    },
                },
                MirValue::RandomValue(u) => {
                    Ok(query_indexed_cur_eval(&mut current_evals.rng, &mut current_evals.rand_values, *u))
                },
                MirValue::PublicInput(pi) => {
                    Ok(query_hashed_cur_eval(&mut current_evals.rng, &mut current_evals.public_inputs, pi))
                },
                MirValue::PeriodicColumn(pc) => {
                    Ok(query_hashed_cur_eval(&mut current_evals.rng, &mut current_evals.periodic_columns, pc))
                },
                MirValue::PublicInputTable(pita) => {
                    Ok(query_hashed_cur_eval(&mut current_evals.rng, &mut current_evals.public_input_tables, pita))
                },
                MirValue::Null
                | MirValue::BusAccess(_)
                | MirValue::Unconstrained
                | MirValue::TraceAccessBinding(_)
                | MirValue::Constant(_) => {
                    // These values are not handled in this function
                    println!("Unexpected values in eval_random_point: {op:?}");
                    Err(CompileError::Failed)
                },
            }
        },
        Op::Accessor(a) => {
            if let Op::Value(v) = a.indexable.borrow().deref() {
                if let MirValue::TraceAccess(trace_access) = v.value.value {
                    // Use accessor offset instead of the trace_access row_offset
                    let index = trace_access.column * 2 + a.offset;
                    match trace_access.segment {
                        0 => {
                            return Ok(query_indexed_cur_eval(
                                &mut current_evals.rng,
                                &mut current_evals.main_trace,
                                index,
                            ));
                        },
                        1 => {
                            return Ok(query_indexed_cur_eval(
                                &mut current_evals.rng,
                                &mut current_evals.aux_trace,
                                index,
                            ));
                        },
                        _ => {
                            println!(
                                "Unexpected segment in eval_random_point: {}",
                                trace_access.segment
                            );
                            return Err(CompileError::Failed);
                        },
                    }
                }
            }
            let indexable = eval_random_point(current_evals, a.indexable.clone())?;
            Ok(indexable)
        },
        Op::Call(_)
        | Op::Fold(_)
        | Op::Boundary(_)
        | Op::For(_)
        | Op::If(_)
        | Op::Vector(_)
        | Op::Matrix(_)
        | Op::None(_) => {
            // These operations are not handled in this function
            println!("Unexpected operation in eval_random_point: {op:?}");
            Err(CompileError::Failed)
        },
    }
}
