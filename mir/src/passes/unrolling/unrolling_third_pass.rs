use std::ops::Deref;

use miden_diagnostics::{DiagnosticsHandler, SourceSpan, Spanned};

use crate::{
    CompileError,
    ir::{
        Add, BackLink, Boundary, ConstantValue, Enf, Exp, FoldOperator, Graph, Link, Matrix,
        MirValue, Mul, Node, Op, Parent, RandomInputs, SpannedMirValue, Sub, TraceAccess,
        TraceAccessBinding, Value, Vector,
    },
    passes::{Visitor, duplicate_node, unrolling::match_optimizer::MatchOptimizer},
};

pub struct UnrollingThirdPass<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,
    // current evaluations of nodes at random points
    random_inputs: RandomInputs,
}

impl<'a> UnrollingThirdPass<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self {
            diagnostics,
            work_stack: vec![],
            random_inputs: RandomInputs::default(),
        }
    }
}

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

fn unroll_binary_op(
    lhs: Link<Op>,
    rhs: Link<Op>,
    parent: Link<Op>,
    span: SourceSpan,
) -> Result<Option<Link<Op>>, CompileError> {
    if let (Op::Vector(lhs_vector), Op::Vector(rhs_vector)) =
        (lhs.borrow().deref(), rhs.borrow().deref())
    {
        let lhs_vec = lhs_vector.children().borrow().deref().clone();
        let rhs_vec = rhs_vector.children().borrow().deref().clone();

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

// For the first pass of Unrolling, we use a tweaked version of the Visitor trait,
// each visit_*_bis function returns an Option<Link<Op>> instead of Result<(), CompileError>,
// to mutate the nodes (e.g. modifying a Operation<Vectors> to Vector<Operations>)
impl UnrollingThirdPass<'_> {
    fn visit_value_bis(
        &mut self,
        _graph: &mut Graph,
        value: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to unwrap because we just dispatched on it
        let value_ref = value.as_value().unwrap();
        let mir_value = value_ref.value.value.clone();
        match &mir_value {
            MirValue::Constant(c) => match c {
                ConstantValue::Felt(_) => Ok(None),
                ConstantValue::Vector(v) => Ok(Some(unroll_constant_vector(v, value_ref.span()))),
                ConstantValue::Matrix(m) => Ok(Some(unroll_constant_matrix(m, value_ref.span()))),
            },
            MirValue::TraceAccessBinding(trace_access_binding) => {
                Ok(Some(unroll_trace_access_binding(trace_access_binding, value_ref.span())))
            },
            MirValue::TraceAccess(_)
            | MirValue::PeriodicColumn(_)
            | MirValue::PublicInput(_)
            | MirValue::PublicInputTable(_)
            | MirValue::RandomValue(_)
            | MirValue::BusAccess(_)
            | MirValue::Null
            | MirValue::Unconstrained => Ok(None),
        }
    }

    fn visit_add_bis(
        &mut self,
        _graph: &mut Graph,
        add: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to unwrap because we just dispatched on it
        let add_ref = add.as_add().unwrap();
        let lhs = add_ref.lhs.clone();
        let rhs = add_ref.rhs.clone();

        unroll_binary_op(lhs, rhs, add.clone(), add_ref.span())
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

        unroll_binary_op(lhs, rhs, sub.clone(), sub_ref.span())
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

        unroll_binary_op(lhs, rhs, mul.clone(), mul_ref.span())
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

        unroll_binary_op(lhs, rhs, exp.clone(), exp_ref.span())
    }

    fn visit_enf_bis(
        &mut self,
        _graph: &mut Graph,
        enf: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let mut updated_enf = None;

        {
            let enf_ref = enf.as_enf().unwrap();
            let expr = enf_ref.expr.clone();
            if let Op::Vector(vec) = expr.borrow().deref() {
                let ops = vec.children().borrow().deref().clone();
                let new_vec =
                    ops.iter().map(|op| Enf::create(op.clone(), enf_ref.span())).collect();
                updated_enf = Some(Vector::create(new_vec, enf_ref.span()));
            };
        }

        Ok(updated_enf)
    }

    fn visit_boundary_bis(
        &mut self,
        _graph: &mut Graph,
        boundary: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to unwrap because we just dispatched on it
        let boundary_ref = boundary.as_boundary().unwrap();
        let expr = boundary_ref.expr.clone();
        let kind = boundary_ref.kind;

        if let Op::Vector(vec) = expr.borrow().deref() {
            let expr_vec = vec.children().borrow().deref().clone();
            let new_vec = expr_vec
                .iter()
                .map(|expr| Boundary::create(expr.clone(), kind, boundary_ref.span()))
                .collect::<Vec<_>>();
            return Ok(Some(Vector::create(new_vec, boundary_ref.span())));
        };

        Ok(None)
    }

    fn visit_fold_bis(
        &mut self,
        _graph: &mut Graph,
        fold: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let fold_ref = fold.as_fold().unwrap();
        let iterator = fold_ref.iterator.clone();
        let operator = fold_ref.operator.clone();
        let initial_value = fold_ref.initial_value.clone();

        let iterator_ref = iterator.borrow();
        let Op::Vector(iterator_vector) = iterator_ref.deref() else {
            unreachable!("Expected vector iterator in fold, found: {:?}", iterator_ref);
        };
        let iterator_nodes = iterator_vector.children().borrow().deref().clone();

        let resulting_node =
            iterator_nodes.iter().fold(initial_value, |acc_node, node| match operator {
                FoldOperator::Add => Add::create(
                    acc_node,
                    duplicate_node(node.clone(), &mut Default::default()),
                    fold_ref.span(),
                ),
                FoldOperator::Mul => Mul::create(
                    acc_node,
                    duplicate_node(node.clone(), &mut Default::default()),
                    fold_ref.span(),
                ),
                FoldOperator::None => {
                    unreachable!("Unexpected unrolling of Fold with None FoldOperator")
                },
            });

        Ok(Some(resulting_node))
    }

    /// Visiting an `If` node consists of evaluating all the main trace constraints contained in
    /// the match arms, and combining them to optimize the resulting vector of constraints if
    /// possible. We handle bus related constraints separately, as they cannot be combined with
    /// main trace constraints.
    fn visit_if_bis(
        &mut self,
        _graph: &mut Graph,
        if_node: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let if_ref = if_node.as_if().unwrap();
        let match_arms = if_ref.match_arms.borrow();

        // 1. Instantiate a new MatchOptimizer to handle the constraints of this node
        let mut match_optimizer = MatchOptimizer::new(&mut self.random_inputs);

        let mut bus_related_constraints = Vec::new();

        // 2. For each match arm, gather bus-related constraints
        // to be handled separately and evaluate the main constraints
        for match_arm in match_arms.iter() {
            let bus_related_constraints_for_match_arm =
                match_optimizer.evaluate_match_arm(match_arm)?;
            bus_related_constraints
                .push((match_arm.condition.clone(), bus_related_constraints_for_match_arm));
        }

        // 3. Construct the new vector of combined main constraints
        let combined_main_constraints = match_optimizer.reduce_main_constraints(if_ref.span);

        // 4. Add all the constraints that are bus-related
        let new_vec = MatchOptimizer::gather_all_constraints(
            &mut bus_related_constraints,
            combined_main_constraints,
            if_ref.span,
        );

        Ok(Some(Vector::create(new_vec, if_ref.span())))
    }

    fn visit_vector_bis(
        &mut self,
        _graph: &mut Graph,
        vector: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to unwrap because we just dispatched on it
        let vector_ref = vector.as_vector().unwrap();
        let children = vector_ref.elements.borrow().deref().clone();
        let size = vector_ref.size;

        if size == 1 {
            let child = children.first().unwrap();
            return Ok(Some(child.clone()));
        }
        Ok(None)
    }

    fn visit_matrix_bis(
        &mut self,
        _graph: &mut Graph,
        _matrix: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        Ok(None) // Matrix are already unrolled, we have nothing to do
    }
}

impl Visitor for UnrollingThirdPass<'_> {
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
        // In this pass, we both need to dispatch the visitor depending on the node type,
        // and also mutate the node if needed. We implement custom visit_*_bis methods
        // that returns a Some(updated_node) if we need to update the node's value.
        let updated_op: Result<Option<Link<Op>>, CompileError> = match node.borrow().deref() {
            Node::Enf(e) => to_link_and(e.clone(), graph, |g, el| self.visit_enf_bis(g, el)),
            Node::Boundary(b) => {
                to_link_and(b.clone(), graph, |g, el| self.visit_boundary_bis(g, el))
            },
            Node::Add(a) => to_link_and(a.clone(), graph, |g, el| self.visit_add_bis(g, el)),
            Node::Sub(s) => to_link_and(s.clone(), graph, |g, el| self.visit_sub_bis(g, el)),
            Node::Mul(m) => to_link_and(m.clone(), graph, |g, el| self.visit_mul_bis(g, el)),
            Node::Exp(e) => to_link_and(e.clone(), graph, |g, el| self.visit_exp_bis(g, el)),
            Node::If(i) => to_link_and(i.clone(), graph, |g, el| self.visit_if_bis(g, el)),
            Node::Fold(f) => to_link_and(f.clone(), graph, |g, el| self.visit_fold_bis(g, el)),
            Node::Vector(v) => to_link_and(v.clone(), graph, |g, el| self.visit_vector_bis(g, el)),
            Node::Matrix(m) => to_link_and(m.clone(), graph, |g, el| self.visit_matrix_bis(g, el)),
            Node::Accessor(_a) => Ok(None),
            Node::BusOp(_b) => Ok(None),
            Node::Value(v) => to_link_and(v.clone(), graph, |g, el| self.visit_value_bis(g, el)),
            Node::None(_) => Ok(None),
            Node::Function(_)
            | Node::Evaluator(_)
            | Node::Call(_)
            | Node::For(_)
            | Node::Parameter(_) => {
                unreachable!(
                    "Unexpected node during Unrolling: Function, Evaluators, Calls, For nodes and Parameters should have been inlined before this pass. Found: {:?}",
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
