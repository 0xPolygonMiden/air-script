use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    ops::{Add, Deref, Mul, Sub},
};

use rand::{distr::Uniform, prelude::*};
use winter_math::{
    FieldElement, StarkField,
    fields::{QuadExtension, f64::BaseElement as Felt},
};

use crate::{
    CompileError,
    ir::{
        ConstantValue, Link, MirValue, Op, PeriodicColumnAccess, PublicInputAccess,
        PublicInputTableAccess,
    },
};

#[derive(Eq, PartialEq, Clone, Debug, Copy)]
pub struct QuadFelt(pub QuadExtension<Felt>);

impl QuadFelt {
    pub fn new_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        // Note: using a uniform distribution over all u64 values would lead to a non-uniform
        // distribution of Felt values.
        let distr = Uniform::new(0, Felt::MODULUS).unwrap(); // Unwrap is safe as Felt::MODULUS is > 0

        QuadFelt(QuadExtension::new(Felt::new(rng.sample(distr)), Felt::new(rng.sample(distr))))
    }

    pub fn new_const(felt: Felt) -> Self {
        QuadFelt(QuadExtension::new(felt, Felt::ZERO))
    }

    /// Returns the integer representation of the evaluations.
    fn as_ints(&self) -> [u64; 2] {
        self.0.to_base_elements().map(|f| Felt::as_int(&f))
    }

    fn exp(
        self,
        power: <QuadExtension<winter_math::fields::f64::BaseElement> as FieldElement>::PositiveInteger,
    ) -> Self {
        QuadFelt(self.0.exp(power))
    }
}

impl Add for QuadFelt {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        QuadFelt(self.0 + other.0)
    }
}

impl Sub for QuadFelt {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        QuadFelt(self.0 - other.0)
    }
}

impl Mul for QuadFelt {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        QuadFelt(self.0 * other.0)
    }
}

impl Hash for QuadFelt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ints().hash(state);
    }
}

impl Ord for QuadFelt {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Here we can reuse the existing comparison implementation for arrays
        self.as_ints().cmp(&other.as_ints())
    }
}

impl PartialOrd for QuadFelt {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents the current existing evaluations to persist random values taken by the same values.
#[derive(Debug, Clone, Default)]
pub struct RandomInputs {
    rng: ThreadRng,
    main_trace: Vec<QuadFelt>,
    aux_trace: Vec<QuadFelt>,
    rand_values: Vec<QuadFelt>,
    public_inputs: HashMap<PublicInputAccess, QuadFelt>,
    periodic_columns: HashMap<PeriodicColumnAccess, QuadFelt>,
    public_input_tables: HashMap<PublicInputTableAccess, QuadFelt>,
}

impl RandomInputs {
    /// Evaluates a given MIR node at random points.
    pub fn eval(&mut self, op: Link<Op>) -> Result<QuadFelt, CompileError> {
        match op.borrow().deref() {
            Op::Enf(e) => {
                let expr = self.eval(e.expr.clone())?;
                Ok(expr)
            },
            Op::Add(a) => {
                let lhs = self.eval(a.lhs.clone())?;
                let rhs = self.eval(a.rhs.clone())?;
                Ok(lhs + rhs)
            },
            Op::Sub(s) => {
                let lhs = self.eval(s.lhs.clone())?;
                let rhs = self.eval(s.rhs.clone())?;
                Ok(lhs - rhs)
            },
            Op::Mul(m) => {
                let lhs = self.eval(m.lhs.clone())?;
                let rhs = self.eval(m.rhs.clone())?;
                Ok(lhs * rhs)
            },
            Op::Exp(e) => {
                let lhs = self.eval(e.lhs.clone())?;
                let rhs = self.eval(e.rhs.clone())?;
                let power = rhs.as_ints()[0];
                Ok(lhs.exp(power))
            },
            Op::BusOp(b) => {
                // For a given bus operation, we hash the operation and its parameters to create a
                // unique identifier for this operation in the extension field,
                // as we haven't expanded the bus constraints yet.
                let mut hasher = DefaultHasher::new();
                "BusOp".hash(&mut hasher);
                b.hash(&mut hasher);
                let hash = hasher.finish();
                let felt = Felt::new(hash);
                Ok(QuadFelt::new_const(felt))
            },
            Op::Parameter(_) => {
                // We cannot easily detect that two parameters refer to the same For, so we consider
                // them to be all different
                Ok(QuadFelt::new_random(&mut self.rng))
            },
            Op::Value(v) => {
                match &v.value.value {
                    MirValue::Constant(ConstantValue::Felt(c)) => {
                        let felt = Felt::new(*c);
                        Ok(QuadFelt::new_const(felt))
                    },
                    MirValue::TraceAccess(trace_access) => match trace_access.segment {
                        0 => {
                            let index = trace_access.column * 2 + trace_access.row_offset;
                            Ok(query_indexed_cur_eval(&mut self.rng, &mut self.main_trace, index))
                        },
                        1 => {
                            let index = trace_access.column * 2 + trace_access.row_offset;
                            Ok(query_indexed_cur_eval(&mut self.rng, &mut self.aux_trace, index))
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
                        Ok(query_indexed_cur_eval(&mut self.rng, &mut self.rand_values, *u))
                    },
                    MirValue::PublicInput(pi) => {
                        Ok(query_hashed_cur_eval(&mut self.rng, &mut self.public_inputs, pi))
                    },
                    MirValue::PeriodicColumn(pc) => {
                        Ok(query_hashed_cur_eval(&mut self.rng, &mut self.periodic_columns, pc))
                    },
                    MirValue::PublicInputTable(pita) => Ok(query_hashed_cur_eval(
                        &mut self.rng,
                        &mut self.public_input_tables,
                        pita,
                    )),
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
                                    &mut self.rng,
                                    &mut self.main_trace,
                                    index,
                                ));
                            },
                            1 => {
                                return Ok(query_indexed_cur_eval(
                                    &mut self.rng,
                                    &mut self.aux_trace,
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
                let indexable = self.eval(a.indexable.clone())?;
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
}

/// Helper function to either query an existing evaluation or create a new random one if the index
/// is out of bounds.
fn query_indexed_cur_eval<R: Rng + ?Sized>(
    rng: &mut R,
    cur_eval_vec: &mut Vec<QuadFelt>,
    index: usize,
) -> QuadFelt {
    if cur_eval_vec.len() <= index {
        cur_eval_vec.resize_with(index + 1, || QuadFelt::new_random(rng));
    }
    cur_eval_vec[index]
}

/// Helper function to either query an existing evaluation or create a new random one if the element
/// is not present in the map.
fn query_hashed_cur_eval<R: Rng + ?Sized, H: Hash + Eq + Clone>(
    rng: &mut R,
    cur_eval_map: &mut HashMap<H, QuadFelt>,
    element: &H,
) -> QuadFelt {
    *cur_eval_map.entry(element.clone()).or_insert_with(|| QuadFelt::new_random(rng))
}
