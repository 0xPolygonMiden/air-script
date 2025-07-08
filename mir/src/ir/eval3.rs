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
pub struct Eval(pub [Felt; NUM_EVALS]);

impl Eval {
    fn exp(self, other: Eval) -> Eval {
        Eval([
            self.0[0].exp(other.0[0].as_int()),
            self.0[1].exp(other.0[1].as_int()),
            self.0[2].exp(other.0[2].as_int()),
        ])
    }

    /// Creates a new [Eval] with random values.
    fn new_random<R: Rng + ?Sized>(rng: &mut R) -> Eval {
        // Note: using a uniform distribution over all u64 values would lead to a non-uniform
        // distribution of Felt values.
        let distr = Uniform::new(0, Felt::MODULUS).unwrap(); // Unwrap is safe as Felt::MODULUS is > 0
        Eval([
            Felt::new(rng.sample(distr)),
            Felt::new(rng.sample(distr)),
            Felt::new(rng.sample(distr)),
        ])
    }
        
    /// Creates a new [Eval] with the same constant value for all evaluations.
    fn new_const(felt: Felt) -> Self {
        Eval([felt; NUM_EVALS])
    }
}

impl Add for Eval {
    type Output = Eval;

    fn add(self, other: Eval) -> Eval {
        Eval([self.0[0] + other.0[0], self.0[1] + other.0[1], self.0[2] + other.0[2]])
    }
}
impl Sub for Eval {
    type Output = Eval;

    fn sub(self, other: Eval) -> Eval {
        Eval([self.0[0] - other.0[0], self.0[1] - other.0[1], self.0[2] - other.0[2]])
    }
}
impl Mul for Eval {
    type Output = Eval;

    fn mul(self, other: Eval) -> Eval {
        Eval([self.0[0] * other.0[0], self.0[1] * other.0[1], self.0[2] * other.0[2]])
    }
}

impl Hash for Eval {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0[0].as_int().hash(state);
        self.0[1].as_int().hash(state);
        self.0[2].as_int().hash(state);
    }
}

impl Ord for Eval {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0[0]
            .as_int()
            .cmp(&other.0[0].as_int())
            .then(self.0[1].as_int().cmp(&other.0[1].as_int()))
            .then(self.0[2].as_int().cmp(&other.0[2].as_int()))
    }
}

impl PartialOrd for Eval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents the current existing evaluations to persist random values taken by the same values.
#[derive(Debug, Clone, Default)]
pub struct CurrentEvals {
    main_trace: Vec<Eval>,
    aux_trace: Vec<Eval>,
    rand_values: Vec<Eval>,
    public_inputs: Vec<Eval>,
}

/// Helper function to either query an existing evaluation or create a new random one if the index
/// is out of bounds.
fn query_cur_eval<R: Rng + ?Sized>(rng: &mut R, cur_eval_vec: &mut Vec<Eval>, index: usize) -> Eval {
    if cur_eval_vec.len() <= index {
        cur_eval_vec.resize_with(index + 1, || Eval::new_random(rng));
    }
    cur_eval_vec[index]
}

/// Evaluates a given MIR node at random points.
pub fn eval_random_point<R: Rng + ?Sized>(
    rng: &mut R,
    current_evals: &mut CurrentEvals,
    op: Link<Op>,
) -> Result<Eval, CompileError> {
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
                            return Ok(query_cur_eval(
                                rng,
                                &mut current_evals.main_trace,
                                index,
                            ));
                        },
                        1 => {
                            return Ok(query_cur_eval(
                                rng,
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
