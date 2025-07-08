use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::{Add, Deref, Mul, Sub},
};

use rand::{distr::Uniform, prelude::*};
use winter_math::{FieldElement, StarkField, fields::f64::BaseElement as Felt};

use crate::{
    CompileError,
    ir::{ConstantValue, Link, MirValue, Op},
};

pub const NUM_EVALS: usize = 3; // Number of random evaluation points

/// A struct representing the evaluation of an operation at `NUM_EVALS` random points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eval<const NUM_EVALS: usize>(pub [Felt; NUM_EVALS]);

impl<const NUM_EVALS: usize> Eval<NUM_EVALS> {
    fn exp(self, other: Self) -> Self {
        Eval(
            self.0
                .iter()
                .zip(other.0.iter())
                .map(|(a, b)| a.exp(b.as_int()))
                .collect::<Vec<Felt>>()
                .try_into()
                .unwrap(),
        )
    }

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
    public_inputs: Vec<Eval<NUM_EVALS>>,
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

/// Evaluates a given MIR node at random points.
pub fn eval_random_point<R: Rng + ?Sized>(
    rng: &mut R,
    current_evals: &mut CurrentEvals<NUM_EVALS>,
    op: Link<Op>,
) -> Result<Eval<NUM_EVALS>, CompileError> {
    match op.borrow().deref() {
        Op::Enf(e) => {
            let expr = eval_random_point(rng, current_evals, e.expr.clone())?;
            Ok(expr)
        },
        Op::Add(a) => {
            let lhs = eval_random_point(rng, current_evals, a.lhs.clone())?;
            let rhs = eval_random_point(rng, current_evals, a.rhs.clone())?;
            Ok(lhs + rhs)
        },
        Op::Sub(s) => {
            let lhs = eval_random_point(rng, current_evals, s.lhs.clone())?;
            let rhs = eval_random_point(rng, current_evals, s.rhs.clone())?;
            Ok(lhs - rhs)
        },
        Op::Mul(m) => {
            let lhs = eval_random_point(rng, current_evals, m.lhs.clone())?;
            let rhs = eval_random_point(rng, current_evals, m.rhs.clone())?;
            Ok(lhs * rhs)
        },
        Op::Exp(e) => {
            let lhs = eval_random_point(rng, current_evals, e.lhs.clone())?;
            let rhs = eval_random_point(rng, current_evals, e.rhs.clone())?;
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
            Ok(Eval::new_random(rng))
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
                        Ok(query_cur_eval(rng, &mut current_evals.main_trace, index))
                    },
                    1 => {
                        let index = trace_access.column * 2 + trace_access.row_offset;
                        Ok(query_cur_eval(rng, &mut current_evals.aux_trace, index))
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
                    Ok(query_cur_eval(rng, &mut current_evals.rand_values, *u))
                },
                MirValue::PublicInput(pi) => {
                    let index = pi.index;
                    Ok(query_cur_eval(rng, &mut current_evals.public_inputs, index))
                },
                MirValue::PeriodicColumn(pc) => {
                    let mut hasher = DefaultHasher::new();
                    "PeriodicColumn".hash(&mut hasher);
                    pc.hash(&mut hasher);
                    let hash = hasher.finish();
                    let felt = Felt::new(hash);
                    Ok(Eval::new_const(felt))
                },
                MirValue::PublicInputTable(pita) => {
                    let mut hasher = DefaultHasher::new();
                    "PublicInputTable".hash(&mut hasher);
                    pita.hash(&mut hasher);
                    let hash = hasher.finish();
                    let felt = Felt::new(hash);
                    Ok(Eval::new_const(felt))
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
                            return Ok(query_cur_eval(rng, &mut current_evals.main_trace, index));
                        },
                        1 => {
                            return Ok(query_cur_eval(rng, &mut current_evals.aux_trace, index));
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
            let indexable = eval_random_point(rng, current_evals, a.indexable.clone())?;
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
