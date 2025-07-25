use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    ops::Deref,
};

use miden_core::{Felt, QuadExtension};
use rand::{distr::Uniform, prelude::*};
use winter_math::{FieldElement, StarkField};

use crate::{
    CompileError,
    ir::{
        ConstantValue, Link, MirValue, Op, PeriodicColumnAccess, PublicInputAccess,
        PublicInputTableAccess,
    },
};

pub type QuadFelt = QuadExtension<Felt>;

/// Returns a random [QuadFelt] value.
fn rand_quad_felt<R: Rng + ?Sized>(rng: &mut R) -> QuadFelt {
    // Note: using a uniform distribution over all u64 values would lead to a non-uniform
    // distribution of Felt values.
    let distr = Uniform::new(0, Felt::MODULUS).unwrap(); // Unwrap is safe as Felt::MODULUS is > 0

    QuadFelt::new(Felt::new(rng.sample(distr)), Felt::new(rng.sample(distr)))
}

/// Returns a [QuadFelt] corresponding to a given base element.
fn const_quad_felt(felt: Felt) -> QuadFelt {
    QuadFelt::new(felt, Felt::ZERO)
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
    ///
    /// Note that we currently assume this will be called only during the unrolling phase, some
    /// operation types are not handled.
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
                let base_elems = rhs.to_base_elements();
                if base_elems[1].as_int() != 0 {
                    // If the second base element is not zero, we cannot evaluate the exponentiation
                    // This should not happen because either the non-constant powers would have been
                    // caught in the parser or it is in the body of a list
                    // comprehension which has not been expanded yet (and we would have a
                    // Op::Parameter instead)
                    Err(CompileError::Failed)
                } else {
                    let power = rhs.to_base_elements()[0].as_int();
                    Ok(lhs.exp(power))
                }
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
                Ok(const_quad_felt(felt))
            },
            Op::Parameter(_) => {
                // We cannot easily detect that two parameters refer to the same For, so we consider
                // them to be all different
                Ok(rand_quad_felt(&mut self.rng))
            },
            Op::Value(v) => {
                match &v.value.value {
                    MirValue::Constant(ConstantValue::Felt(c)) => {
                        let felt = Felt::new(*c);
                        Ok(const_quad_felt(felt))
                    },
                    // For each trace segment, we associate a random value to the each trace access
                    // indexed in the following way, each column having two
                    // distinct evaluations to account for the two possible row offsets:
                    // $main[0], $main[0]', $main[1], $main[1]', $main[2], ...
                    // Note: if we encounter a trace access corresponding to an index we have not
                    // yet evaluated, we will randomly generate values for
                    // this trace access, but also for all previous indices.
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
                                "Unexpected segment in RandomInputs::eval: {}",
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
                        println!("Unexpected values in RandomInputs::eval: {op:?}");
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
                                    "Unexpected segment in RandomInputs::eval: {}",
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
                // These operations are not handled in this function, as we currently expect this
                // function to be called only on during Unrolling.
                println!("Unexpected operation in RandomInputs::eval: {op:?}");
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
        cur_eval_vec.resize_with(index + 1, || rand_quad_felt(rng));
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
    *cur_eval_map.entry(element.clone()).or_insert_with(|| rand_quad_felt(rng))
}
