extern crate alloc;
use alloc::collections::BTreeMap;
use std::ops::Deref;

use air_parser::ast::TraceSegmentId;
use miden_core::{Felt, QuadExtension};
use rand::{distr::Uniform, prelude::*};
use winter_math::{FieldElement, StarkField};

use crate::{
    CompileError,
    ir::{
        ConstantValue, Link, MirAccessType, MirValue, Op, Parent, PeriodicColumnAccess,
        PublicInputAccess, TraceAccess,
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
pub fn const_quad_felt(felt: Felt) -> QuadFelt {
    QuadFelt::new(felt, Felt::ZERO)
}

/// Helper function to either query an existing evaluation or create a new random one if the index
/// is out of bounds.
pub fn query_indexed_eval<R: Rng + ?Sized>(
    rng: &mut R,
    evaluations: &mut Vec<QuadFelt>,
    index: usize,
) -> QuadFelt {
    if evaluations.len() <= index {
        evaluations.resize_with(index + 1, || rand_quad_felt(rng));
    }
    evaluations[index]
}

/// Helper function to either query an existing evaluation or create a new random one if the element
/// is not present in the map.
pub fn query_mapped_eval<R: Rng + ?Sized, K: Ord + Eq + Clone>(
    rng: &mut R,
    evaluation_map: &mut BTreeMap<K, QuadFelt>,
    element: &K,
) -> QuadFelt {
    *evaluation_map.entry(element.clone()).or_insert_with(|| rand_quad_felt(rng))
}

/// Holds the random inputs taken by leaf nodes, in order to persist them across different node
/// evaluations.
#[derive(Debug, Clone, Default)]
pub struct RandomInputs {
    rng: ThreadRng,
    // A vector to hold the random values taken for the main trace, indexed in the following way:
    // $main[0], $main[0]', $main[1], $main[1]', $main[2], ...
    main_trace: Vec<QuadFelt>,
    rand_values: Vec<QuadFelt>,
    public_inputs: BTreeMap<PublicInputAccess, QuadFelt>,
    periodic_columns: BTreeMap<PeriodicColumnAccess, QuadFelt>,
}

impl RandomInputs {
    fn const_index(&self, node: &Link<Op>, context: &str) -> Result<usize, CompileError> {
        if let Op::Value(value) = node.borrow().deref()
            && let MirValue::Constant(ConstantValue::Felt(c)) = value.value.value
        {
            return Ok(c as usize);
        }
        println!("Non-constant index in RandomInputs::eval for {context}");
        Err(CompileError::Failed)
    }

    fn eval_trace_access(&mut self, trace_access: TraceAccess) -> Result<QuadFelt, CompileError> {
        let index = trace_access.column * 2 + trace_access.row_offset;
        match trace_access.segment {
            TraceSegmentId::Main => {
                Ok(query_indexed_eval(&mut self.rng, &mut self.main_trace, index))
            },
            _ => {
                println!(
                    "Unexpected trace_access segment in RandomInputs::eval: {}. This segment should only be used for buses and should be handled separately.",
                    trace_access.segment
                );
                Err(CompileError::Failed)
            },
        }
    }

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
                    // We associate a random value to each trace access of the main trace,
                    // indexed in the following way, each column having two
                    // distinct evaluations to account for the two possible row offsets:
                    // $main[0], $main[0]', $main[1], $main[1]', $main[2], ...
                    // Note: if we encounter a trace access corresponding to an index we have not
                    // yet evaluated, we will randomly generate values for
                    // this trace access, but also for all previous indices.
                    MirValue::TraceAccess(trace_access) => self.eval_trace_access(*trace_access),
                    MirValue::RandomValue(u) => {
                        Ok(query_indexed_eval(&mut self.rng, &mut self.rand_values, *u))
                    },
                    // For PublicInput and PeriodicColumn, we use the Hash of the element to
                    // associate a unique random value or each public input and
                    // each periodic column access
                    MirValue::PublicInput(pi) => {
                        Ok(query_mapped_eval(&mut self.rng, &mut self.public_inputs, pi))
                    },
                    MirValue::PeriodicColumn(pc) => {
                        Ok(query_mapped_eval(&mut self.rng, &mut self.periodic_columns, pc))
                    },
                    MirValue::Null
                    | MirValue::BusAccess(_)
                    | MirValue::Unconstrained
                    | MirValue::PublicInputTable(_) => {
                        // Bus related values, we expect these to be handled separately
                        println!(
                            "Unexpected values in RandomInputs::eval, the following op should only be used for Bus expressions and should be handled separately: {op:?}"
                        );
                        Err(CompileError::Failed)
                    },
                    MirValue::TraceAccessBinding(_) | MirValue::Constant(_) => {
                        // These should have been unrolled as Vector and handled beforehand
                        println!(
                            "Unexpected values in RandomInputs::eval, the following op should already be Unrolled at this stage: {op:?}"
                        );
                        Err(CompileError::Failed)
                    },
                }
            },
            Op::Accessor(a) => match &a.access_type {
                MirAccessType::Default => {
                    if let Op::Value(v) = a.indexable.borrow().deref() {
                        match v.value.value {
                            MirValue::TraceAccess(trace_access) => {
                                let trace_access = TraceAccess {
                                    row_offset: trace_access.row_offset + a.offset,
                                    ..trace_access
                                };
                                return self.eval_trace_access(trace_access);
                            },
                            MirValue::PublicInput(public_input_access) => {
                                let public_input_access = PublicInputAccess {
                                    index: public_input_access.index + a.offset,
                                    ..public_input_access
                                };
                                return Ok(query_mapped_eval(
                                    &mut self.rng,
                                    &mut self.public_inputs,
                                    &public_input_access,
                                ));
                            },
                            _ => {},
                        }
                    }
                    let indexable = self.eval(a.indexable.clone())?;
                    Ok(indexable)
                },
                MirAccessType::Index(index) => {
                    let index_usize = self.const_index(index, "index accessor")?;
                    match a.indexable.borrow().deref() {
                        Op::Vector(vector) => {
                            let children = vector.children();
                            let elements = children.borrow();
                            let child =
                                elements.get(index_usize).ok_or(CompileError::Failed)?.clone();
                            self.eval(child)
                        },
                        Op::Value(v) => match v.value.value {
                            MirValue::PublicInput(public_input_access) => {
                                let public_input_access = PublicInputAccess {
                                    index: public_input_access.index + index_usize,
                                    ..public_input_access
                                };
                                Ok(query_mapped_eval(
                                    &mut self.rng,
                                    &mut self.public_inputs,
                                    &public_input_access,
                                ))
                            },
                            MirValue::TraceAccess(trace_access) => {
                                let trace_access = TraceAccess {
                                    column: trace_access.column + index_usize,
                                    ..trace_access
                                };
                                self.eval_trace_access(trace_access)
                            },
                            _ => {
                                println!(
                                    "Unexpected indexable value in RandomInputs::eval: {op:?}"
                                );
                                Err(CompileError::Failed)
                            },
                        },
                        _ => {
                            println!(
                                "Unexpected indexable in RandomInputs::eval for index accessor: {op:?}"
                            );
                            Err(CompileError::Failed)
                        },
                    }
                },
                MirAccessType::Matrix(row, col) => {
                    let row_index = self.const_index(row, "matrix row accessor")?;
                    let col_index = self.const_index(col, "matrix col accessor")?;
                    match a.indexable.borrow().deref() {
                        Op::Matrix(matrix) => {
                            let rows_link = matrix.children();
                            let rows = rows_link.borrow();
                            let row_node = rows.get(row_index).ok_or(CompileError::Failed)?.clone();
                            if let Some(row_vec) = row_node.as_vector() {
                                let cols_link = row_vec.children();
                                let cols = cols_link.borrow();
                                let col_node =
                                    cols.get(col_index).ok_or(CompileError::Failed)?.clone();
                                self.eval(col_node)
                            } else {
                                self.eval(row_node)
                            }
                        },
                        _ => {
                            println!(
                                "Unexpected indexable in RandomInputs::eval for matrix accessor: {op:?}"
                            );
                            Err(CompileError::Failed)
                        },
                    }
                },
            },
            Op::Call(_) => {
                // We expect Inlining to have already been done before Unrolling
                println!(
                    "Unexpected operation in RandomInputs::eval, Calls should have been Inlined in a previous pass: {op:?}"
                );
                Err(CompileError::Failed)
            },
            Op::Fold(_) | Op::Vector(_) | Op::Matrix(_) | Op::For(_) | Op::If(_) | Op::None(_) => {
                println!(
                    "Unexpected operation in RandomInputs::eval, the following operation should already be Unrolled at this stage: {op:?}"
                );
                Err(CompileError::Failed)
            },
            Op::Boundary(_) => {
                println!(
                    "Unexpected operation in RandomInputs::eval, the following operation is not valid in integrity constraints: {op:?}"
                );
                Err(CompileError::Failed)
            },
            Op::BusOp(_) => {
                // Bus related operation, we expect these to be handled separately
                println!(
                    "Unexpected operation in RandomInputs::eval, the following op should only be used for Bus expressions and should be handled separately: {op:?}"
                );
                Err(CompileError::Failed)
            },
        }
    }
}
