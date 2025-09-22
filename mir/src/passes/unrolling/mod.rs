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

/// This pass follows a similar approach as the Inlining pass and requires that the latter has
/// already been done.
///
/// * In the first step, we visit the graph, unrolling each node type except `For` nodes. Instead,
///   for these node types we gather the context to inline them in the second pass. In this first
///   pass, we also optimize constraints found in match statements.
/// * In the second pass, we inline the bodies of `For` nodes.
pub struct Unrolling<'a> {
    diagnostics: &'a DiagnosticsHandler,
}

impl<'a> Unrolling<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}

/// This structure is used to keep track of what is needed to inline a For node
#[derive(Clone, Debug)]
pub struct ForInliningContext {
    body: Link<Op>,
    iterators: Vec<Link<Op>>,
    selector: Option<Link<Op>>,
    ref_node: Link<Op>,
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
        let mut second_pass = UnrollingSecondPass::new(
            self.diagnostics,
            first_pass.bodies_to_inline.clone(),
            first_pass.all_for_nodes.clone(),
        );
        Visitor::run(&mut second_pass, ir.constraint_graph_mut())?;

        // The third pass unrolls all the remaining nodes (`If` nodes and their parents)
        let mut third_pass = UnrollingThirdPass::new(self.diagnostics);
        Visitor::run(&mut third_pass, ir.constraint_graph_mut())?;
        Ok(ir)
    }
}

/// Unrolls an `Enf` on vectors into a `Vector<Enf>`.
pub fn visit_enf_bis(enf: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    let enf_ref = enf.as_enf().unwrap();
    let expr = enf_ref.expr.clone();
    if let Op::Vector(vec) = expr.borrow().deref() {
        let ops = vec.children().borrow().clone();
        let new_vec = ops.iter().map(|op| Enf::create(op.clone(), enf_ref.span())).collect();
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

/// Unrolls an `Add` on vectors into vectors of `Add`.
pub fn visit_add_bis(add: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    // safe to unwrap because we just dispatched on it
    let add_ref = add.as_add().unwrap();
    let lhs = add_ref.lhs.clone();
    let rhs = add_ref.rhs.clone();
    unroll_binary_op(lhs, rhs, add.clone(), add_ref.span())
}

/// Unrolls a `Sub` on vectors into vectors of `Sub`.
pub fn visit_sub_bis(sub: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    // safe to unwrap because we just dispatched on it
    let sub_ref = sub.as_sub().unwrap();
    let lhs = sub_ref.lhs.clone();
    let rhs = sub_ref.rhs.clone();
    unroll_binary_op(lhs, rhs, sub.clone(), sub_ref.span())
}

/// Unrolls a `Mul` on vectors into vectors of `Mul`.
pub fn visit_mul_bis(mul: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    // safe to unwrap because we just dispatched on it
    let mul_ref = mul.as_mul().unwrap();
    let lhs = mul_ref.lhs.clone();
    let rhs = mul_ref.rhs.clone();
    unroll_binary_op(lhs, rhs, mul.clone(), mul_ref.span())
}

/// Unrolls an `Exp` on vectors into vectors of `Exp`.
pub fn visit_exp_bis(exp: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    // safe to unwrap because we just dispatched on it
    let exp_ref = exp.as_exp().unwrap();
    let lhs = exp_ref.lhs.clone();
    let rhs = exp_ref.rhs.clone();
    unroll_binary_op(lhs, rhs, exp.clone(), exp_ref.span())
}

/// Unrolls a `Boundary` on vectors into a `Vector<Boundary>`.
pub fn visit_boundary_bis(boundary: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    // safe to unwrap because we just dispatched on it
    let boundary_ref = boundary.as_boundary().unwrap();
    let expr = boundary_ref.expr.clone();
    let kind = boundary_ref.kind;
    if let Op::Vector(vec) = expr.borrow().deref() {
        let expr_vec = vec.children().borrow().clone();
        let mut new_vec = vec![];
        for expr in expr_vec.iter() {
            let new_node = Boundary::create(expr.clone(), kind, boundary_ref.span());
            new_vec.push(new_node);
        }
        return Ok(Some(Vector::create(new_vec, boundary_ref.span())));
    };
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

/// Unrolls a binary operation (`Add`, `Sub`, `Mul`, `Exp`) on vectors into vectors of binary
/// operations.
fn unroll_binary_op(
    lhs: Link<Op>,
    rhs: Link<Op>,
    parent: Link<Op>,
    span: SourceSpan,
) -> Result<Option<Link<Op>>, CompileError> {
    if let (Op::Vector(lhs_vector), Op::Vector(rhs_vector)) =
        (lhs.borrow().deref(), rhs.borrow().deref())
    {
        let lhs_vec = lhs_vector.children().borrow().clone();
        let rhs_vec = rhs_vector.children().borrow().clone();

        if lhs_vec.len() != rhs_vec.len() {
            unreachable!("Binary operation children type mismatch: {:?}", parent);
        } else {
            let mut new_vec = vec![];
            for (lhs, rhs) in lhs_vec.iter().zip(rhs_vec.iter()) {
                let new_node = match parent.borrow().deref() {
                    Op::Add(_) => Add::create(lhs.clone(), rhs.clone(), span),
                    Op::Sub(_) => Sub::create(lhs.clone(), rhs.clone(), span),
                    Op::Mul(_) => Mul::create(lhs.clone(), rhs.clone(), span),
                    Op::Exp(_) => Exp::create(lhs.clone(), rhs.clone(), span),
                    _ => unreachable!("Unexpected parent operation: {:?}", parent),
                };
                new_vec.push(new_node);
            }
            return Ok(Some(Vector::create(new_vec, parent.span())));
        }
    }

    Ok(None)
}
