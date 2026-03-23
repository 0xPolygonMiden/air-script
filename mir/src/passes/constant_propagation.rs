use std::ops::Deref;

use air_pass::Pass;
use miden_diagnostics::{DiagnosticsHandler, Severity, SourceSpan, Spanned};

use super::visitor::Visitor;
use crate::{
    CompileError,
    ir::{
        ConstantValue, Graph, Link, Mir, MirAccessType, MirValue, Node, Op, Parent,
        SpannedMirValue, Value,
    },
    passes::{handle_accessor_visit, should_skip_accessor_unroll},
};

pub struct ConstantPropagation<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,
    work_stack: Vec<Link<Node>>,
}

impl Pass for ConstantPropagation<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        Visitor::run(self, ir.constraint_graph_mut())?;
        Ok(ir)
    }
}

impl<'a> ConstantPropagation<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics, work_stack: vec![] }
    }
}

// For the ConstantPropagation, we use a tweaked version of the Visitor trait,
// each visit_*_bis function returns an Option<Link<Op>> instead of Result<(), CompileError>,
// to mutate the nodes (e.g. modifying a Add(lhs, rhs) to Value(lhs + rhs)).
impl ConstantPropagation<'_> {
    fn visit_add_bis(&mut self, add: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
        let Some(add_ref) = add.as_add() else {
            return Ok(None);
        };
        let lhs = add_ref.lhs.clone();
        let rhs = add_ref.rhs.clone();

        if let Some(0) = get_inner_const(&lhs) {
            Ok(Some(rhs))
        } else if let Some(0) = get_inner_const(&rhs) {
            Ok(Some(lhs))
        } else {
            try_fold_const_binary_op(lhs, rhs, add.clone(), add_ref.span())
        }
    }

    fn visit_sub_bis(&mut self, sub: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
        let Some(sub_ref) = sub.as_sub() else {
            return Ok(None);
        };
        let lhs = sub_ref.lhs.clone();
        let rhs = sub_ref.rhs.clone();

        if let Some(0) = get_inner_const(&rhs) {
            Ok(Some(lhs))
        } else {
            try_fold_const_binary_op(lhs, rhs, sub.clone(), sub_ref.span())
        }
    }

    fn visit_mul_bis(&mut self, mul: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
        let Some(mul_ref) = mul.as_mul() else {
            return Ok(None);
        };
        let lhs = mul_ref.lhs.clone();
        let rhs = mul_ref.rhs.clone();

        match (get_inner_const(&lhs), get_inner_const(&rhs)) {
            (Some(0), _) | (_, Some(0)) => Ok(Some(Value::create(SpannedMirValue {
                value: MirValue::Constant(ConstantValue::Felt(0)),
                span: mul_ref.span,
            }))),
            (Some(1), _) => Ok(Some(rhs)),
            (_, Some(1)) => Ok(Some(lhs)),
            _ => try_fold_const_binary_op(lhs, rhs, mul.clone(), mul_ref.span()),
        }
    }

    fn visit_exp_bis(&mut self, exp: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
        let Some(exp_ref) = exp.as_exp() else {
            return Ok(None);
        };
        let lhs = exp_ref.lhs.clone();
        let rhs = exp_ref.rhs.clone();

        // x^0 = 1 (must take precedence, including 0^0)
        if let Some(0) = get_inner_const(&rhs) {
            Ok(Some(Value::create(SpannedMirValue {
                value: MirValue::Constant(ConstantValue::Felt(1)),
                span: exp_ref.span,
            })))
        } else if let Some(0) = get_inner_const(&lhs) {
            // 0^k = 0, but only when k is known and non-zero
            if get_inner_const(&rhs).is_some() {
                return Ok(Some(Value::create(SpannedMirValue {
                    value: MirValue::Constant(ConstantValue::Felt(0)),
                    span: exp_ref.span,
                })));
            }
            Ok(None)
        } else {
            try_fold_const_binary_op(lhs, rhs, exp.clone(), exp_ref.span())
        }
    }

    fn visit_accessor_bis(&mut self, accessor: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
        let Some(accessor_ref) = accessor.as_accessor() else {
            return Ok(None);
        };
        if should_skip_accessor_unroll(&accessor_ref.indexable) {
            let ensure_const_index = |index: &Link<Op>| -> Result<(), CompileError> {
                if get_inner_const(index).is_some() {
                    return Ok(());
                }
                self.diagnostics
                    .diagnostic(Severity::Error)
                    .with_message("the index is not constant during constant propagation")
                    .with_primary_label(index.span(), "index is not constant")
                    .emit();
                Err(CompileError::Failed)
            };
            match &accessor_ref.access_type {
                MirAccessType::Index(index) => {
                    ensure_const_index(index)?;
                },
                MirAccessType::Matrix(row, col) => {
                    ensure_const_index(row)?;
                    ensure_const_index(col)?;
                },
                MirAccessType::Default => {},
            }
            return Ok(None);
        }
        handle_accessor_visit(accessor.clone(), true, self.diagnostics)
    }
}

impl Visitor for ConstantPropagation<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }

    // We visit all boundary constraints and all integrity constraints
    // No need to visit the functions or evaluators, as they should have been inlined before this
    // pass
    fn root_nodes_to_visit(&self, graph: &Graph) -> Vec<Link<Node>> {
        let boundary_constraints_roots_ref = graph.boundary_constraints_roots.borrow();
        let integrity_constraints_roots_ref = graph.integrity_constraints_roots.borrow();
        let bus_roots: Vec<_> = graph
            .buses
            .values()
            .flat_map(|b| b.borrow().clone().columns.into_iter().collect::<Vec<_>>())
            .collect();
        let combined_roots = boundary_constraints_roots_ref
            .clone()
            .into_iter()
            .map(|bc| bc.as_node())
            .chain(integrity_constraints_roots_ref.clone().into_iter().map(|ic| ic.as_node()))
            .chain(bus_roots.into_iter().map(|b| b.as_node()));
        combined_roots.collect()
    }

    fn visit_node(&mut self, _graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        if node.is_stale() {
            return Ok(());
        }

        // In this pass, we both need to dispatch the visitor depending on the node type,
        // and also mutate the node if needed. We implement custom visit_*_bis methods
        // that returns a Some(updated_node) if we need to update the node's value.
        let updated_op: Option<Link<Op>> = match node.borrow().deref() {
            Node::Add(a) => a.to_link().map_or(Ok(None), |el| self.visit_add_bis(el))?,
            Node::Sub(s) => s.to_link().map_or(Ok(None), |el| self.visit_sub_bis(el))?,
            Node::Mul(m) => m.to_link().map_or(Ok(None), |el| self.visit_mul_bis(el))?,
            Node::Exp(e) => e.to_link().map_or(Ok(None), |el| self.visit_exp_bis(el))?,
            Node::Accessor(e) => e.to_link().map_or(Ok(None), |el| self.visit_accessor_bis(el))?,
            Node::Vector(_)
            | Node::Matrix(_)
            | Node::Enf(_)
            | Node::Boundary(_)
            | Node::BusOp(_)
            | Node::Value(_)
            | Node::None(_) => None,
            _ => {
                unreachable!(
                    "Unexpected node during Mir's ConstantPropagation: Function, Evaluators, Calls, If, For, Fold and Parameter should have been inlined and unrolled before this pass. Found: {:?}",
                    node
                );
            },
        };

        // We update the node if needed
        if let Some(updated_op) = updated_op {
            node.as_op().unwrap().set(&updated_op);
        }

        Ok(())
    }
}

// HELPERS FUNCTIONS
// ================================================================================================

pub fn get_inner_const(value: &Link<Op>) -> Option<u64> {
    match value.borrow().deref() {
        Op::Value(v) => v.get_inner_const(),
        Op::Accessor(accessor) => {
            match (accessor.access_type.clone(), accessor.indexable.borrow().deref()) {
                (MirAccessType::Default, _) => get_inner_const(&accessor.indexable),
                (MirAccessType::Index(index), Op::Vector(vector)) => {
                    let index = get_inner_const(&index).expect("Expected constant index") as usize;

                    let vec_children = vector.children();
                    let vec_ref = vec_children.borrow();
                    vec_ref.get(index).and_then(get_inner_const)
                },
                (MirAccessType::Matrix(row, col), Op::Matrix(matrix)) => {
                    let row = get_inner_const(&row).expect("Expected constant row") as usize;
                    let col = get_inner_const(&col).expect("Expected constant column") as usize;

                    let mat_children = matrix.children();
                    let mat_ref = mat_children.borrow();
                    mat_ref.get(row).and_then(|row| {
                        if let Op::Vector(row_vector) = row.borrow().deref() {
                            let row_children = row_vector.children();
                            let row_ref = row_children.borrow();
                            row_ref.get(col).and_then(get_inner_const)
                        } else {
                            None
                        }
                    })
                },
                _ => None,
            }
        },
        _ => None,
    }
}

/// Helper function to fold constant binary operations (Add, Sub, Mul, Exp)
/// into their resulting value if both operands are constant values.
fn try_fold_const_binary_op(
    lhs: Link<Op>,
    rhs: Link<Op>,
    parent: Link<Op>,
    span: SourceSpan,
) -> Result<Option<Link<Op>>, CompileError> {
    let mut updated_binary_op = None;

    if let (Some(lhs_const), Some(rhs_const)) = (get_inner_const(&lhs), get_inner_const(&rhs)) {
        let folded = match parent.borrow().deref() {
            Op::Add(_) => lhs_const.checked_add(rhs_const),
            Op::Sub(_) => lhs_const.checked_sub(rhs_const),
            Op::Mul(_) => lhs_const.checked_mul(rhs_const),
            Op::Exp(_) => {
                let rhs_const = rhs_const.try_into().map_err(|_| CompileError::Failed)?;
                lhs_const.checked_pow(rhs_const)
            },
            _ => unreachable!("Unexpected parent operation: {:?}", parent),
        };
        if let Some(folded) = folded {
            let new_value = Value::create(SpannedMirValue {
                value: MirValue::Constant(crate::ir::ConstantValue::Felt(folded)),
                span,
            });
            updated_binary_op = Some(new_value);
        }
    }

    Ok(updated_binary_op)
}
