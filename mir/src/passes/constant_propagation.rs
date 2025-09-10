use std::ops::Deref;

use air_parser::ast::AccessType;
use air_pass::Pass;
use miden_diagnostics::{DiagnosticsHandler, SourceSpan, Spanned};

use super::visitor::Visitor;
use crate::{
    CompileError,
    ir::{
        BackLink, ConstantValue, Graph, Link, Mir, MirValue, Node, Op, Parent, SpannedMirValue,
        Value,
    },
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
    fn visit_add_bis(
        &mut self,
        _graph: &mut Graph,
        add: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to unwrap because we just dispatched on it
        let add_ref = add.as_add().unwrap();
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

    fn visit_sub_bis(
        &mut self,
        _graph: &mut Graph,
        sub: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to unwrap because we just dispatched on it
        let sub_ref = sub.as_sub().unwrap();
        let lhs = sub_ref.lhs.clone();
        let rhs = sub_ref.rhs.clone();

        if let Some(0) = get_inner_const(&rhs) {
            Ok(Some(lhs))
        } else {
            try_fold_const_binary_op(lhs, rhs, sub.clone(), sub_ref.span())
        }
    }

    fn visit_mul_bis(
        &mut self,
        _graph: &mut Graph,
        mul: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to unwrap because we just dispatched on it
        let mul_ref = mul.as_mul().unwrap();
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

    fn visit_exp_bis(
        &mut self,
        _graph: &mut Graph,
        exp: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to unwrap because we just dispatched on it
        let exp_ref = exp.as_exp().unwrap();
        let lhs = exp_ref.lhs.clone();
        let rhs = exp_ref.rhs.clone();

        if let Some(0) = get_inner_const(&lhs) {
            Ok(Some(Value::create(SpannedMirValue {
                value: MirValue::Constant(ConstantValue::Felt(0)),
                span: exp_ref.span,
            })))
        } else if let Some(0) = get_inner_const(&rhs) {
            Ok(Some(Value::create(SpannedMirValue {
                value: MirValue::Constant(ConstantValue::Felt(1)),
                span: exp_ref.span,
            })))
        } else {
            try_fold_const_binary_op(lhs, rhs, exp.clone(), exp_ref.span())
        }
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

    fn visit_node(&mut self, graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        if node.is_stale() {
            return Ok(());
        }

        // In this pass, we both need to dispatch the visitor depending on the node type,
        // and also mutate the node if needed. We implement custom visit_*_bis methods
        // that returns a Some(updated_node) if we need to update the node's value.
        let updated_op: Result<Option<Link<Op>>, CompileError> = match node.borrow().deref() {
            Node::Add(a) => to_link_and(a.clone(), graph, |g, el| self.visit_add_bis(g, el)),
            Node::Sub(s) => to_link_and(s.clone(), graph, |g, el| self.visit_sub_bis(g, el)),
            Node::Mul(m) => to_link_and(m.clone(), graph, |g, el| self.visit_mul_bis(g, el)),
            Node::Exp(e) => to_link_and(e.clone(), graph, |g, el| self.visit_exp_bis(g, el)),
            // For all the following cases, there is nothing to fold
            Node::Vector(_)
            | Node::Matrix(_)
            | Node::Enf(_)
            | Node::Boundary(_)
            | Node::BusOp(_)
            | Node::Value(_)
            | Node::Accessor(_)
            | Node::None(_) => Ok(None),
            Node::Function(_) | Node::Evaluator(_) | Node::Call(_) => {
                unreachable!(
                    "Unexpected node during Mir's ConstantPropagation: Function, Evaluators and Calls should have been inlined before this pass. Found: {:?}",
                    node
                );
            },
            Node::If(_) | Node::For(_) | Node::Fold(_) | Node::Parameter(_) => {
                unreachable!(
                    "Unexpected node during Mir's ConstantPropagation: If, For, Fold and Parameter should have been unrolled before this pass. Found: {:?}",
                    node
                );
            },
        };

        // We update the node if needed
        if let Some(updated_op) = updated_op? {
            node.as_op().unwrap().set(&updated_op);
        }

        Ok(())
    }
}

// HELPERS FUNCTIONS
// ================================================================================================

/// Tries to upgrade a BackLink to a Link<Op> and apply a given closure to it if it is successful,
/// otherwise returns None.
fn to_link_and<F>(
    back: BackLink<Op>,
    graph: &mut Graph,
    f: F,
) -> Result<Option<Link<Op>>, CompileError>
where
    F: FnOnce(&mut Graph, Link<Op>) -> Result<Option<Link<Op>>, CompileError>,
{
    if let Some(op) = back.to_link() {
        f(graph, op)
    } else {
        Ok(None)
    }
}

/// Helper function to extract the constant felt value from a Link<Op> if it is one.
fn get_inner_const(value: &Link<Op>) -> Option<u64> {
    match value.borrow().deref() {
        Op::Value(Value {
            value:
                SpannedMirValue {
                    value: MirValue::Constant(ConstantValue::Felt(c)),
                    ..
                },
            ..
        }) => Some(*c),
        Op::Accessor(accessor) => {
            match (accessor.access_type.clone(), accessor.indexable.borrow().deref()) {
                (AccessType::Default, _) => get_inner_const(&accessor.indexable),
                (AccessType::Index(index), Op::Vector(vector)) => {
                    let vec_children = vector.children();
                    let vec_ref = vec_children.borrow();
                    vec_ref.get(index).and_then(get_inner_const)
                },
                (AccessType::Matrix(row, col), Op::Matrix(matrix)) => {
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
