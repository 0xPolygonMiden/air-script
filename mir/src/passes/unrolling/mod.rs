//! MIR unrolling pass.
//!
//! The goal is to expand comprehensions and control-flow into explicit constraints. We do this in
//! three passes: (1) unroll non-`For` nodes, (2) inline `For` bodies while preserving parameter
//! identity, and (3) finish remaining `If`/match constructs. The tradeoff is potential graph
//! growth, so we rely on earlier simplifications and prioritize correctness over minimal size.

use std::ops::Deref;

use air_pass::Pass;
use miden_diagnostics::{DiagnosticsHandler, SourceSpan, Spanned};

use super::visitor::Visitor;
use crate::{CompileError, ir::*};

mod match_optimizer;
mod unrolling_first_pass;
mod unrolling_second_pass;
mod unrolling_third_pass;

use unrolling_first_pass::UnrollingFirstPass;
use unrolling_second_pass::UnrollingSecondPass;
use unrolling_third_pass::UnrollingThirdPass;

/// Unrolls constraints and comprehensions after inlining.
///
/// It runs in three stages:
/// 1. Unrolls everything except `For` nodes and records their contexts.
/// 2. Inlines `For` bodies using the recorded contexts.
/// 3. Unrolls remaining `If` nodes and optimizes match constraints.
pub struct Unrolling<'a> {
    diagnostics: &'a DiagnosticsHandler,
}

impl<'a> Unrolling<'a> {
    /// Construct a new unrolling pass.
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}

/// Context needed to inline a `For` node.
#[derive(Clone, Debug)]
pub struct ForInliningContext {
    body: Link<Op>,
    iterators: Vec<Link<Op>>,
    selector: Option<Link<Op>>,
    ref_owner_id: OwnerId,
}

impl Pass for Unrolling<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        // The first pass unrolls all nodes fully, except for:
        // - `For` nodes
        // - `If` nodes and their parents
        let mut first_pass = UnrollingFirstPass::new(self.diagnostics);
        Visitor::run(&mut first_pass, ir.constraint_graph_mut())?;

        // The second pass actually inlines the `For` nodes
        let mut second_pass =
            UnrollingSecondPass::new(self.diagnostics, first_pass.bodies_to_inline.clone());
        Visitor::run(&mut second_pass, ir.constraint_graph_mut())?;

        // The third pass unrolls all the remaining nodes (`If` nodes and their parents)
        let mut third_pass = UnrollingThirdPass::new(self.diagnostics);
        third_pass.run(ir.constraint_graph_mut())?;
        Ok(ir)
    }
}

/// Unrolls an `Enf` on vectors into a `Vector<Enf>`.
pub fn visit_enf_bis(enf: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    let enf_ref = enf.as_enf().unwrap();
    if enf_ref.tag.is_some() {
        return Ok(None);
    }
    let expr = enf_ref.expr.clone();
    if let Op::Vector(vec) = expr.borrow().deref() {
        let ops = vec.children().borrow().clone();
        let new_vec = ops.iter().map(|op| Enf::create(op.clone(), enf_ref.span(), None)).collect();
        return Ok(Some(Vector::create(new_vec, enf_ref.span())));
    }
    Ok(None)
}

/// Unrolls a `Value` if it represents a constant vector or matrix, or a trace access binding.
pub fn visit_value_bis(value: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    // safe to unwrap because we just dispatched on it
    let value_ref = value.as_value().unwrap();
    let mir_value = value_ref.value.value.clone();
    match &mir_value {
        MirValue::Constant(c) => match c {
            ConstantValue::Felt(_) => {},
            ConstantValue::Vector(v) => {
                return Ok(Some(unroll_constant_vector(v, value_ref.span())));
            },
            ConstantValue::Matrix(m) => {
                return Ok(Some(unroll_constant_matrix(m, value_ref.span())));
            },
        },
        MirValue::TraceAccessBinding(trace_access_binding) => {
            return Ok(Some(unroll_trace_access_binding(trace_access_binding, value_ref.span())));
        },
        MirValue::TraceAccess(_)
        | MirValue::PeriodicColumn(_)
        | MirValue::PublicInput(_)
        | MirValue::PublicInputTable(_)
        | MirValue::RandomValue(_)
        | MirValue::BusAccess(_)
        | MirValue::Null
        | MirValue::Unconstrained => {},
    }
    Ok(None)
}

/// Unrolls a `Fold`. We replace the `Fold` node by a series of binary operations (e.g. `Add` or
/// `Mul`) applied to the initial value and each element of the iterator, depending on the
/// `FoldOperator`.
pub fn visit_fold_bis(fold: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    let fold_ref = fold.as_fold().unwrap();
    let iterator = fold_ref.iterator.clone();
    let operator = fold_ref.operator.clone();
    let initial_value = fold_ref.initial_value.clone();
    let iterator_ref = iterator.borrow();
    let Op::Vector(iterator_vector) = iterator_ref.deref() else {
        unreachable!("Expected vector iterator in fold, found: {:?}", iterator_ref);
    };
    let iterator_nodes = iterator_vector.children().borrow().clone();
    let resulting_node =
        iterator_nodes.iter().fold(initial_value, |acc_node, node| match operator {
            FoldOperator::Add => Add::create(acc_node, node.clone(), fold_ref.span()),
            FoldOperator::Mul => Mul::create(acc_node, node.clone(), fold_ref.span()),
            FoldOperator::None => {
                unreachable!("Unexpected unrolling of Fold with None FoldOperator")
            },
        });
    Ok(Some(resulting_node))
}

/// Unrolls a `Vector`. As vectors are already unrolled, we only need to replace vectors of size 1
/// by their only child (i.e. a scalar).
pub fn visit_vector_bis(vector: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    // safe to unwrap because we just dispatched on it
    let vector_ref = vector.as_vector().unwrap();
    let children = vector_ref.elements.borrow().clone();
    let size = vector_ref.size;

    // If the vector is of size 1, it is a scalar and we replace it by its only child
    if size == 1 {
        let child = children.first().unwrap();
        return Ok(Some(child.clone()));
    }
    // Otherwise, it is already in its unrolled form, we do nothing
    Ok(None)
}

// PRIVATE HELPER FUNCTIONS
// ================================================================================================

/// Unrolls a trace access binding into either:
/// - a TraceAccess if it is of size 1
/// - or a Vector<TraceAccess> otherwise.
fn unroll_trace_access_binding(
    trace_access_binding: &TraceAccessBinding,
    span: SourceSpan,
) -> Link<Op> {
    if trace_access_binding.size == 1 {
        Value::create(SpannedMirValue {
            span,
            value: MirValue::TraceAccess(TraceAccess {
                segment: trace_access_binding.segment,
                column: trace_access_binding.offset,
                row_offset: 0,
            }),
        })
    } else {
        let mut vec = vec![];
        for index in 0..trace_access_binding.size {
            let val = Value::create(SpannedMirValue {
                span,
                value: MirValue::TraceAccess(TraceAccess {
                    segment: trace_access_binding.segment,
                    column: trace_access_binding.offset + index,
                    row_offset: 0,
                }),
            });
            vec.push(val);
        }
        Vector::create(vec, span)
    }
}

/// Unrolls a constant vector into a Vector<ConstantValue::Felt>`
fn unroll_constant_vector(constant_vector: &Vec<u64>, span: SourceSpan) -> Link<Op> {
    let mut vec = vec![];
    for val in constant_vector {
        let val = Value::create(SpannedMirValue {
            span,
            value: MirValue::Constant(ConstantValue::Felt(*val)),
        });
        vec.push(val);
    }
    Vector::create(vec, span)
}

/// Unrolls a constant matrix into a `Matrix<Vector<ConstantValue::Felt>>`
fn unroll_constant_matrix(constant_matrix: &Vec<Vec<u64>>, span: SourceSpan) -> Link<Op> {
    let mut res_m = vec![];
    for row in constant_matrix {
        let mut res_row = vec![];
        for val in row {
            let val = Value::create(SpannedMirValue {
                span,
                value: MirValue::Constant(ConstantValue::Felt(*val)),
            });
            res_row.push(val);
        }
        let res_row_vec = Vector::create(res_row, span);
        res_m.push(res_row_vec);
    }
    Matrix::create(res_m, span)
}
