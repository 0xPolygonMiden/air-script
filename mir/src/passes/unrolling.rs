use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
    rc::Rc,
    vec,
};

use air_pass::Pass;
use miden_diagnostics::{DiagnosticsHandler, Spanned};

use super::{duplicate_node_or_replace, visitor::Visitor};
use crate::{
    CompileError,
    ir::*,
    passes::{duplicate_node, get_inner_const},
};

/// This pass follows a similar approach as the Inlining pass.
/// It requires that this Inlining pass has already been done.
///
/// * In the first step, we visit the graph, unrolling each node type except For nodes. Instead, for
///   these node types we gather the context to inline them in the second pass.
/// * In the second pass, we inline the bodies of For nodes.
///
/// TODO:
/// - [ ] Implement diagnostics for better error handling
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
impl ForInliningContext {}

pub struct UnrollingFirstPass<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,

    // current evaluations of nodes at random points
    random_inputs: RandomInputs,

    // For each child of a For node encountered, we store the context to inline it in the second
    // pass
    bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,

    // We keep track of all parameters referencing a given For node
    params_for_ref_node: HashMap<usize, Vec<Link<Op>>>,
    // We keep a reference to For nodes in order to avoid the backlinks stored in Parameters
    // referencing them to be dropped
    all_for_nodes: HashMap<usize, (Link<Op>, Link<Owner>)>,
}

impl<'a> UnrollingFirstPass<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self {
            diagnostics,
            work_stack: vec![],
            random_inputs: RandomInputs::default(),
            bodies_to_inline: vec![],
            params_for_ref_node: HashMap::new(),
            all_for_nodes: HashMap::new(),
        }
    }
}

pub struct UnrollingSecondPass<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,
    // A list of all the children of For nodes to inline
    bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,
    // The current context for inlining a For node, if any
    for_inlining_context: Option<ForInliningContext>,
    // A map of nodes to replace with their inlined version
    nodes_to_replace: HashMap<usize, (Link<Op>, Link<Op>)>,
    // We keep track of all parameters referencing a given For node
    params_for_ref_node: HashMap<usize, Vec<Link<Op>>>,
    // We keep a reference to For nodes in order to avoid the backlinks stored in Parameters
    // referencing them to be dropped
    all_for_nodes: HashMap<usize, (Link<Op>, Link<Owner>)>,
}
impl<'a> UnrollingSecondPass<'a> {
    pub fn new(
        diagnostics: &'a DiagnosticsHandler,
        bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,
        all_for_nodes: HashMap<usize, (Link<Op>, Link<Owner>)>,
    ) -> Self {
        Self {
            diagnostics,
            work_stack: vec![],
            bodies_to_inline,
            for_inlining_context: None,
            nodes_to_replace: HashMap::new(),
            params_for_ref_node: HashMap::new(),
            all_for_nodes,
        }
    }
}

impl Pass for Unrolling<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        // The first pass unrolls all nodes fully, except for For nodes
        let mut first_pass = UnrollingFirstPass::new(self.diagnostics);
        Visitor::run(&mut first_pass, ir.constraint_graph_mut())?;

        // The second pass actually inlines the For nodes
        let mut second_pass = UnrollingSecondPass::new(
            self.diagnostics,
            first_pass.bodies_to_inline.clone(),
            first_pass.all_for_nodes.clone(),
        );
        Visitor::run(&mut second_pass, ir.constraint_graph_mut())?;

        Ok(ir)
    }
}

// For the first pass of Unrolling, we use a tweaked version of the Visitor trait,
// each visit_*_bis function returns an Option<Link<Op>> instead of Result<(), CompileError>,
// to mutate the nodes (e.g. modifying a Operation<Vectors> to Vector<Operations>)
impl UnrollingFirstPass<'_> {
    fn visit_value_bis(
        &mut self,
        _graph: &mut Graph,
        value: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to unwrap because we just dispatched on it
        let mut updated_value = None;

        {
            let value_ref = value.as_value().unwrap();
            let mir_value = value_ref.value.value.clone();
            match mir_value {
                MirValue::Constant(c) => match c {
                    ConstantValue::Felt(_) => {},
                    ConstantValue::Vector(v) => {
                        let mut vec = vec![];
                        for val in v {
                            let val = Value::create(SpannedMirValue {
                                span: value_ref.value.span,
                                value: MirValue::Constant(ConstantValue::Felt(val)),
                            });
                            vec.push(val);
                        }
                        updated_value = Some(Vector::create(vec, value_ref.span()));
                    },
                    ConstantValue::Matrix(m) => {
                        let mut res_m = vec![];
                        for row in m {
                            let mut res_row = vec![];
                            for val in row {
                                let val = Value::create(SpannedMirValue {
                                    span: value_ref.value.span,
                                    value: MirValue::Constant(ConstantValue::Felt(val)),
                                });
                                res_row.push(val);
                            }
                            let res_row_vec = Vector::create(res_row, value_ref.span());
                            res_m.push(res_row_vec);
                        }
                        updated_value = Some(Matrix::create(res_m, value_ref.span()));
                    },
                },
                MirValue::TraceAccess(_) => {},
                MirValue::PeriodicColumn(_) => {},
                MirValue::PublicInput(_) => {},
                MirValue::PublicInputTable(_) => {},
                MirValue::RandomValue(_) => {},
                MirValue::TraceAccessBinding(trace_access_binding) => {
                    // Create Trace Access based on this binding
                    if trace_access_binding.size == 1 {
                        let val = Value::create(SpannedMirValue {
                            span: value_ref.value.span,
                            value: MirValue::TraceAccess(TraceAccess {
                                segment: trace_access_binding.segment,
                                column: trace_access_binding.offset,
                                row_offset: 0, // ???
                            }),
                        });
                        updated_value = Some(val);
                    } else {
                        let mut vec = vec![];
                        for index in 0..trace_access_binding.size {
                            let val = Value::create(SpannedMirValue {
                                span: value_ref.span(),
                                value: MirValue::TraceAccess(TraceAccess {
                                    segment: trace_access_binding.segment,
                                    column: trace_access_binding.offset + index,
                                    row_offset: 0, // ???
                                }),
                            });
                            vec.push(val);
                        }
                        updated_value = Some(Vector::create(vec, value_ref.span()));
                    }
                },
                MirValue::BusAccess(_) => {},
                MirValue::Null => {},
                MirValue::Unconstrained => {},
            }
        }

        Ok(updated_value)
    }

    fn visit_add_bis(
        &mut self,
        _graph: &mut Graph,
        add: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to un wrap because we just dispatched on it

        let mut updated_add = None;

        {
            let add_ref = add.as_add().unwrap();
            let lhs = add_ref.lhs.clone();
            let rhs = add_ref.rhs.clone();

            if let (Op::Vector(lhs_vector), Op::Vector(rhs_vector)) =
                (lhs.borrow().deref(), rhs.borrow().deref())
            {
                let lhs_vec = lhs_vector.children().borrow().deref().clone();
                let rhs_vec = rhs_vector.children().borrow().deref().clone();

                if lhs_vec.len() != rhs_vec.len() {
                    // Raise diag
                    todo!();
                } else {
                    let mut new_vec = vec![];
                    for (lhs, rhs) in lhs_vec.iter().zip(rhs_vec.iter()) {
                        let new_node = Add::create(lhs.clone(), rhs.clone(), add_ref.span());
                        new_vec.push(new_node);
                    }
                    updated_add = Some(Vector::create(new_vec, add_ref.span()));
                }
            };
        }

        Ok(updated_add)
    }

    fn visit_sub_bis(
        &mut self,
        _graph: &mut Graph,
        sub: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to unwrap because we just dispatched on it

        let mut updated_sub = None;

        let sub_ref = sub.as_sub().unwrap();
        let lhs = sub_ref.lhs.clone();
        let rhs = sub_ref.rhs.clone();

        if let (Op::Vector(lhs_vector), Op::Vector(rhs_vector)) =
            (lhs.borrow().deref(), rhs.borrow().deref())
        {
            let lhs_vec = lhs_vector.children().borrow().deref().clone();
            let rhs_vec = rhs_vector.children().borrow().deref().clone();

            if lhs_vec.len() != rhs_vec.len() {
                // Raise diag
            } else {
                let mut new_vec = vec![];
                for (lhs, rhs) in lhs_vec.iter().zip(rhs_vec.iter()) {
                    let new_node = Sub::create(lhs.clone(), rhs.clone(), sub_ref.span());
                    new_vec.push(new_node);
                }
                updated_sub = Some(Vector::create(new_vec, sub_ref.span()));
            }
        };

        Ok(updated_sub)
    }

    fn visit_mul_bis(
        &mut self,
        _graph: &mut Graph,
        mul: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let mut updated_mul = None;

        {
            let mul_ref = mul.as_mul().unwrap();
            let lhs = mul_ref.lhs.clone();
            let rhs = mul_ref.rhs.clone();

            if let (Op::Vector(lhs_vector), Op::Vector(rhs_vector)) =
                (lhs.borrow().deref(), rhs.borrow().deref())
            {
                let lhs_vec = lhs_vector.children().borrow().deref().clone();
                let rhs_vec = rhs_vector.children().borrow().deref().clone();

                if lhs_vec.len() != rhs_vec.len() {
                    // Raise diag
                } else {
                    let mut new_vec = vec![];
                    for (lhs, rhs) in lhs_vec.iter().zip(rhs_vec.iter()) {
                        let new_node = Mul::create(lhs.clone(), rhs.clone(), mul_ref.span());
                        new_vec.push(new_node);
                    }
                    updated_mul = Some(Vector::create(new_vec, mul_ref.span()));
                }
            };
        }

        Ok(updated_mul)
    }

    fn visit_exp_bis(
        &mut self,
        _graph: &mut Graph,
        exp: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let mut updated_exp = None;

        {
            let exp_ref = exp.as_exp().unwrap();
            let lhs = exp_ref.lhs.clone();
            let rhs = exp_ref.rhs.clone();

            if let (Op::Vector(lhs_vector), Op::Vector(rhs_vector)) =
                (lhs.borrow().deref(), rhs.borrow().deref())
            {
                let lhs_vec = lhs_vector.children().borrow().deref().clone();
                let rhs_vec = rhs_vector.children().borrow().deref().clone();

                if lhs_vec.len() != rhs_vec.len() {
                    // Raise diag
                } else {
                    let mut new_vec = vec![];
                    for (lhs, rhs) in lhs_vec.iter().zip(rhs_vec.iter()) {
                        let new_node = Exp::create(lhs.clone(), rhs.clone(), exp_ref.span());
                        new_vec.push(new_node);
                    }
                    updated_exp = Some(Vector::create(new_vec, exp_ref.span()));
                }
            };
        }

        Ok(updated_exp)
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
                let mut new_vec = vec![];
                for op in ops.iter() {
                    let new_node = Enf::create(op.clone(), enf_ref.span());
                    new_vec.push(new_node);
                }
                updated_enf = Some(Vector::create(new_vec, enf_ref.span()));
            };
        }

        Ok(updated_enf)
    }

    fn visit_fold_bis(
        &mut self,
        _graph: &mut Graph,
        fold: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let updated_fold;

        {
            let fold_ref = fold.as_fold().unwrap();
            let iterator = fold_ref.iterator.clone();
            let operator = fold_ref.operator.clone();
            let initial_value = fold_ref.initial_value.clone();

            let iterator_ref = iterator.borrow();
            let Op::Vector(iterator_vector) = iterator_ref.deref() else {
                unreachable!();
            };
            let iterator_nodes = iterator_vector.children().borrow().deref().clone();

            let mut acc_node = initial_value;
            match operator {
                FoldOperator::Add => {
                    for iterator_node in iterator_nodes {
                        let new_acc_node = Add::create(acc_node, iterator_node, fold_ref.span());
                        acc_node = new_acc_node;
                    }
                },
                FoldOperator::Mul => {
                    for iterator_node in iterator_nodes {
                        let new_acc_node = Mul::create(acc_node, iterator_node, fold_ref.span());
                        acc_node = new_acc_node;
                    }
                },
                FoldOperator::None => {},
            }
            updated_fold = Some(acc_node);
        }

        Ok(updated_fold)
    }

    fn visit_parameter_bis(
        &mut self,
        _graph: &mut Graph,
        parameter: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // FIXME: Just check that the parameter is a scalar, raise diag otherwise
        // List comprehension bodies should only be scalar expressions
        let owner_ref = parameter
            .as_parameter()
            .unwrap()
            .ref_node
            .to_link()
            .unwrap_or_else(|| panic!("Ref node invalid"));

        self.params_for_ref_node
            .entry(owner_ref.get_ptr())
            .or_default()
            .push(parameter.clone());
        Ok(None)
    }

    fn visit_if_bis(
        &mut self,
        _graph: &mut Graph,
        if_node: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let if_ref = if_node.as_if().unwrap();
        let match_arms = if_ref.match_arms.borrow();

        let mut bus_related_constraints = Vec::new();

        // 1. We evaluate all constraints of this match node at random points
        let mut node_evals = Vec::new();

        // Used to keep track of the evaluation of each constraint
        // We use indices to the node_evals vector as keys, to avoid non-determinism for
        // iterating across
        let mut constraints_evaluation_indices: BTreeMap<usize, Vec<_>> = BTreeMap::new();
        for match_arm in match_arms.iter() {
            let condition = match_arm.condition.clone();
            let expr = match_arm.expr.clone();

            // 1.1. Get all the individual constraints corresponding to this arm
            let all_constraints = if let Op::Vector(expr_vector) = expr.borrow().deref() {
                expr_vector.children().borrow().deref().clone()
            } else {
                vec![expr.clone()]
            };
            let mut constraints_to_eval = Vec::new();
            let mut bus_related_constraints_for_match_arm = Vec::new();

            // 1.2. Filter out all BusOp nodes from the constraints, we will handle them separately
            for constraint in all_constraints {
                match constraint.borrow().deref() {
                    Op::BusOp(_) => bus_related_constraints_for_match_arm.push(constraint.clone()),
                    Op::Enf(enf) => match enf.expr.borrow().deref() {
                        Op::BusOp(_) => {
                            bus_related_constraints_for_match_arm.push(enf.expr.clone())
                        },
                        _ => constraints_to_eval.push(constraint.clone()),
                    },
                    _ => constraints_to_eval.push(constraint.clone()),
                }
            }
            bus_related_constraints
                .push((condition.clone(), bus_related_constraints_for_match_arm));

            // 1.3. Evaluate all the other constraints at random points
            for constraint in constraints_to_eval {
                let eval = self.random_inputs.eval(constraint.clone())?;
                // Check if we already have this eval in our list
                match node_evals.iter().position(|e| e == &eval) {
                    Some(index) => {
                        // If the eval is already in the list, we use its index in the
                        // node_evals vec
                        let Some(constraints_vec) = constraints_evaluation_indices.get_mut(&index)
                        else {
                            unreachable!(
                                "Error: Reference to an evaluation index not found in constraints_evaluation_indices"
                            );
                        };
                        // We add the constraint to the corresponding vector only if the
                        // condition is not already present
                        // to remove duplicate constraints for the same selector
                        if !constraints_vec.iter().any(|(c, _)| c == &condition) {
                            constraints_vec.push((condition.clone(), constraint.clone()))
                        }
                    },
                    None => {
                        // Otherwise, we add it to the list and use its index
                        node_evals.push(eval);
                        let index = node_evals.len() - 1;
                        if constraints_evaluation_indices
                            .insert(index, vec![(condition.clone(), constraint.clone())])
                            .is_some()
                        {
                            unreachable!(
                                "Error: A new evaluation index was found in constraints_evaluation_indices"
                            );
                        }
                    },
                }
            }
        }

        // 2. For each evaluation, we keep track of the length of the vector of unique constraints
        //    evaluating to it
        // This len will be used to combine the constraints by taking the most constrained
        // evaluations first, in order to minimize the number of constraints in the
        // final vector

        // eval_lens: BTreeMap<len, Vec<evaluation_index>>
        let mut eval_lens = BTreeMap::new();
        for (eval_index, constraints_vec) in constraints_evaluation_indices.iter() {
            eval_lens
                .entry(constraints_vec.len())
                .and_modify(|v: &mut Vec<_>| v.push(*eval_index))
                .or_insert(vec![*eval_index]);
        }

        // 3. Construct the new vector of combined constraints
        let mut new_vec = vec![];

        // Note: this will always terminate as we always remove at least one constraint per
        // iteration
        while !constraints_evaluation_indices.is_empty() {
            let mut new_constraint = vec![];
            let mut taken_eval_indices = vec![];

            // Start picking constraints from the biggest sets that evaluate to the same values
            for eval_index in eval_lens.values().cloned().rev().flatten() {
                let constraints = constraints_evaluation_indices.get(&eval_index).unwrap();

                if have_disjoint_conditions(new_constraint.clone(), constraints) {
                    // If the new constraints are disjoint from the current ones, we can add
                    // them
                    new_constraint.push(constraints.clone());
                    taken_eval_indices.push(eval_index);
                }
            }

            // Remove the picked evaluation indices from all structures
            for eval_index in taken_eval_indices {
                constraints_evaluation_indices.remove(&eval_index);
                eval_lens.iter_mut().for_each(|(_len, evals)| {
                    evals.retain(|&e_idx| e_idx != eval_index);
                });
                eval_lens.retain(|_, constraints| !constraints.is_empty());
            }

            // Create combined constraint
            let mut cur_node = None;
            for equivalent_constraints in new_constraint {
                let mut cur_condition = None;
                for (condition, _) in equivalent_constraints.clone() {
                    if cur_condition.is_none() {
                        cur_condition = Some(condition.clone());
                    } else {
                        cur_condition = Some(Add::create(
                            cur_condition.unwrap(),
                            condition.clone(),
                            if_ref.span(),
                        ));
                    }
                }
                let new_node = Mul::create(
                    cur_condition.unwrap(),
                    equivalent_constraints.first().unwrap().1.clone(),
                    if_ref.span(),
                );

                cur_node = match cur_node {
                    Some(existing) => Some(Add::create(existing, new_node, if_ref.span())),
                    None => Some(new_node),
                };
            }

            // The following unwrap is safe as we always have at least one constraint above
            new_vec.push(cur_node.unwrap());
        }

        // 4. Add all the constraints that are bus-related
        for (condition, constraints) in bus_related_constraints.iter_mut() {
            for constraint in constraints.iter_mut() {
                let cur_latch = constraint.as_bus_op().unwrap().latch.clone();
                let new_latch = Mul::create(
                    condition.clone(),
                    duplicate_node(cur_latch, &mut HashMap::new()),
                    if_ref.span(),
                );
                constraint
                    .as_bus_op_mut()
                    .unwrap()
                    .latch
                    .borrow_mut()
                    .clone_from(&new_latch.borrow());
                new_vec.push(constraint.clone());
            }
        }

        Ok(Some(Vector::create(new_vec, if_ref.span())))
    }

    fn visit_boundary_bis(
        &mut self,
        _graph: &mut Graph,
        boundary: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let mut updated_boundary = None;

        {
            // safe to unwrap because we just dispatched on it
            let boundary_ref = boundary.as_boundary().unwrap();
            let expr = boundary_ref.expr.clone();
            let kind = boundary_ref.kind;

            if let Op::Vector(vec) = expr.borrow().deref() {
                let expr_vec = vec.children().borrow().deref().clone();
                let mut new_vec = vec![];
                for expr in expr_vec.iter() {
                    let new_node = Boundary::create(expr.clone(), kind, boundary_ref.span());
                    new_vec.push(new_node);
                }
                updated_boundary = Some(Vector::create(new_vec, boundary_ref.span()));
            };
        }

        Ok(updated_boundary)
    }

    fn visit_accessor_bis(
        &mut self,
        _graph: &mut Graph,
        accessor: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let mut updated_accessor = None;

        {
            let accessor_ref = accessor.as_accessor().unwrap();
            let indexable = accessor_ref.indexable.clone();
            let mir_access_type = accessor_ref.access_type.clone();
            let offset = accessor_ref.offset;

            // FIXME: Factorize this code with the visit_accessor_bis of ConstantPropagation pass
            // We should be able to call a helper, and specify whether we expect the indices to be
            // constants at this stage
            if indexable.clone().as_parameter().is_none() {
                match mir_access_type {
                    MirAccessType::Default => {
                        updated_accessor = Some(indexable.clone());

                        if let Some(value) = indexable.clone().as_value() {
                            let mir_value = value.value.value.clone();

                            if let MirValue::TraceAccess(trace_access) = mir_value {
                                let new_node = Value::create(SpannedMirValue {
                                    span: value.value.span(),
                                    value: MirValue::TraceAccess(TraceAccess {
                                        segment: trace_access.segment,
                                        column: trace_access.column,
                                        row_offset: trace_access.row_offset + offset,
                                    }),
                                });
                                updated_accessor = Some(new_node);
                            }
                        }
                    },
                    MirAccessType::Index(index) => {
                        let index_usize = match get_inner_const(&index) {
                            Some(value) => value as usize,
                            None => {
                                return Ok(None);
                            },
                        };
                        // Check that the child node is a vector, raise diag otherwise
                        // Replace the current node by the index-th element of the vector
                        // Raise diag if index is out of bounds
                        if let Op::Vector(indexable_vector) = indexable.borrow().deref() {
                            let indexable_vec =
                                indexable_vector.children().borrow().deref().clone();
                            let child_accessed = match indexable_vec.get(index_usize) {
                                Some(child_accessed) => child_accessed,
                                None => {
                                    self.diagnostics
                                        .diagnostic(miden_diagnostics::Severity::Error)
                                        .with_message(
                                            "attempted to access an index which is out of bounds",
                                        )
                                        .with_primary_label(index.span(), "index out of bounds")
                                        .emit();
                                    return Err(CompileError::Failed);
                                },
                            };
                            if let Some(value) = child_accessed.clone().as_value() {
                                let mir_value = value.value.value.clone();
                                match mir_value {
                                    MirValue::TraceAccess(trace_access) => {
                                        let new_node = Value::create(SpannedMirValue {
                                            span: value.value.span(),
                                            value: MirValue::TraceAccess(TraceAccess {
                                                segment: trace_access.segment,
                                                column: trace_access.column,
                                                row_offset: trace_access.row_offset + offset,
                                            }),
                                        });
                                        updated_accessor = Some(new_node);
                                    },
                                    _ => {
                                        updated_accessor = Some(child_accessed.clone());
                                    },
                                }
                            } else {
                                updated_accessor = Some(child_accessed.clone());
                            }
                        } else if let Some(value) = indexable.clone().as_value() {
                            let mir_value = value.value.value.clone();
                            match mir_value {
                                MirValue::PublicInput(public_input_access) => {
                                    let new_node = Value::create(SpannedMirValue {
                                        span: value.value.span(),
                                        value: MirValue::PublicInput(PublicInputAccess {
                                            name: public_input_access.name,
                                            index: public_input_access.index + index_usize,
                                        }),
                                    });
                                    updated_accessor = Some(new_node);
                                },
                                MirValue::TraceAccess(trace_access) => {
                                    let new_node = Value::create(SpannedMirValue {
                                        span: value.value.span(),
                                        value: MirValue::TraceAccess(TraceAccess {
                                            segment: trace_access.segment,
                                            column: trace_access.column + index_usize,
                                            row_offset: trace_access.row_offset,
                                        }),
                                    });
                                    updated_accessor = Some(new_node);
                                },
                                _ => {
                                    unreachable!("indexable is {:?}", indexable); // raise diag
                                },
                            }
                        } else {
                            unreachable!("indexable is {:?}", indexable); // raise diag
                        }
                    },
                    MirAccessType::Matrix(row, col) => {
                        let (row, col) = match (get_inner_const(&row), get_inner_const(&col)) {
                            (Some(row), Some(col)) => (row as usize, col as usize),
                            _ => {
                                return Ok(None);
                            },
                        };
                        // Check that the child node is a matrix, raise diag otherwise
                        // Replace the current node by the index-th element of the vector
                        // Raise diag if index is out of bounds

                        if let Op::Vector(indexable_vector) = indexable.borrow().deref() {
                            let indexable_vec =
                                indexable_vector.children().borrow().deref().clone();
                            let row_accessed = match indexable_vec.get(row) {
                                Some(row_accessed) => row_accessed,
                                None => unreachable!(), // raise diag
                            };

                            if let Op::Vector(row_accessed_vector) = row_accessed.borrow().deref() {
                                let row_accessed_vec =
                                    row_accessed_vector.children().borrow().deref().clone();
                                let child_accessed = match row_accessed_vec.get(col) {
                                    Some(child_accessed) => child_accessed,
                                    None => unreachable!(), // raise diag
                                };
                                updated_accessor = Some(child_accessed.clone());
                            } else {
                                unreachable!(); // raise diag
                            };
                        } else if let Op::Matrix(indexable_matrix) = indexable.borrow().deref() {
                            let indexable_vec =
                                indexable_matrix.children().borrow().deref().clone();
                            let row_accessed = match indexable_vec.get(row) {
                                Some(row_accessed) => row_accessed,
                                None => unreachable!(), // raise diag
                            };

                            if let Op::Vector(row_accessed_vector) = row_accessed.borrow().deref() {
                                let row_accessed_vec =
                                    row_accessed_vector.children().borrow().deref().clone();
                                let child_accessed = match row_accessed_vec.get(col) {
                                    Some(child_accessed) => child_accessed,
                                    None => unreachable!(), // raise diag
                                };
                                updated_accessor = Some(child_accessed.clone());
                            } else {
                                unreachable!(); // raise diag
                            };
                        };
                    },
                }
            }
        }

        Ok(updated_accessor)
    }

    fn compute_iterator_len(iterator: Link<Op>) -> usize {
        match iterator.borrow().deref() {
            Op::Vector(vector) => vector.size,
            Op::Matrix(matrix) => matrix.size,
            Op::Accessor(accessor) => match &accessor.access_type {
                MirAccessType::Default => Self::compute_iterator_len(accessor.indexable.clone()),
                MirAccessType::Index(_) => match accessor.indexable.borrow().deref() {
                    Op::Vector(_) => 1,
                    Op::Matrix(matrix) => {
                        let children = matrix.children().borrow().deref().clone();
                        match children.first() {
                            Some(first_row) => match first_row.as_vector() {
                                Some(row_vector) => row_vector.size,
                                _ => unreachable!(), // Raise diag
                            },
                            None => {
                                unreachable!(); // Raise diag
                            },
                        }
                    },
                    _ => unreachable!(), // Raise diag
                },
                MirAccessType::Matrix(..) => 1,
            },
            Op::Parameter(parameter) => match parameter.ty {
                MirType::Felt => 1,
                MirType::Vector(l) => l,
                MirType::Matrix(l, _) => l,
            },
            _ => 1,
        }
    }

    fn visit_for_bis(
        &mut self,
        _graph: &mut Graph,
        for_node: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let updated_for;

        {
            // For each value produced by the iterators, we need to:
            // - Duplicate the body
            // - Visit the body and replace the Variables with the value (with the correct index
            //   depending on the binding)
            // If there is a selector, we need to enforce the selector on the body through an if
            // node ?

            let for_node_clone = for_node.clone();
            let for_ref = for_node_clone.as_for().unwrap();
            let iterators_ref = for_ref.iterators.borrow();
            let iterators = iterators_ref.deref();
            let expr = for_ref.expr.clone();
            let selector = for_ref.selector.clone();

            // Check iterator lengths
            if iterators.is_empty() {
                unreachable!(); // Raise diag
            }
            let iterator_expected_len = Self::compute_iterator_len(iterators[0].clone());

            for iterator in iterators.iter().skip(1) {
                let iterator_len = Self::compute_iterator_len(iterator.clone());
                if iterator_len != iterator_expected_len {
                    unreachable!()
                    // Raise diag
                }
            }

            let mut new_vec = vec![];

            for i in 0..iterator_expected_len {
                let new_node =
                    Parameter::create(i, MirType::Felt, for_node.as_for().unwrap().deref().span());
                new_vec.push(new_node.clone());

                let iterators_i = iterators
                    .iter()
                    .map(|op| {
                        match op.borrow().deref() {
                            Op::Vector(vector) => {
                                let children = vector.children().borrow().deref().clone();
                                children[i].clone()
                            },
                            Op::Matrix(matrix) => {
                                let children = matrix.children().borrow().deref().clone();
                                children[i].clone()
                            },
                            Op::Accessor(accessor) => {
                                match accessor.indexable.borrow().deref() {
                                    // If we access an outer loop parameter in the body of an inner
                                    // loop, we need to create
                                    // an Accessor for the correct index in this parameter
                                    Op::Parameter(_parameter) => {
                                        let mir_access_type =
                                            MirAccessType::Index(Value::create(SpannedMirValue {
                                                span: accessor.span(),
                                                value: MirValue::Constant(ConstantValue::Felt(
                                                    i as u64,
                                                )),
                                            }));
                                        Accessor::create(
                                            accessor.indexable.clone(),
                                            mir_access_type,
                                            0,
                                            accessor.span(),
                                        )
                                    },
                                    _ => op.clone(),
                                }
                            },
                            _ => op.clone(),
                        }
                    })
                    .collect::<Vec<_>>();
                let selector = if let Op::None(_) = selector.borrow().deref() {
                    None
                } else {
                    Some(selector.clone())
                };

                self.bodies_to_inline.push((
                    new_node.clone(),
                    ForInliningContext {
                        body: expr.clone(),
                        iterators: iterators_i,
                        selector,
                        ref_node: for_node.clone(),
                    },
                ));
            }

            let new_vec_op = Vector::create(new_vec.clone(), for_node.span());
            for param in new_vec {
                param.as_parameter_mut().unwrap().set_ref_node(new_vec_op.as_owner().unwrap());
            }
            updated_for = Some(new_vec_op);
        }

        Ok(updated_for)
    }

    fn visit_call_bis(
        &mut self,
        _graph: &mut Graph,
        _call: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        unreachable!("Calls should have been inlined before this pass");
    }

    fn visit_vector_bis(
        &mut self,
        _graph: &mut Graph,
        vector: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let mut updated_vector = None;

        {
            // safe to unwrap because we just dispatched on it
            let vector_ref = vector.as_vector().unwrap();
            let children = vector_ref.elements.borrow().deref().clone();
            let size = vector_ref.size;

            if size == 1 {
                let child = children.first().unwrap();
                updated_vector = Some(child.clone());
            }
        }

        Ok(updated_vector)
        //Ok(None)
    }
    fn visit_matrix_bis(
        &mut self,
        _graph: &mut Graph,
        _matrix: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        Ok(None)
    }
}

impl Visitor for UnrollingFirstPass<'_> {
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
        // We keep a reference to all For nodes to avoid dropping the backlinks stored in Parameters
        if let Some(owner) = node.clone().as_owner()
            && let Some(op) = owner.clone().as_op()
            && let Some(_for_node) = op.as_for()
        {
            self.all_for_nodes.insert(op.get_ptr(), (op.clone(), owner.clone()));
        }

        let updated_op: Result<Option<Link<Op>>, CompileError> = match node.borrow().deref() {
            Node::Function(_f) => {
                unreachable!("Functions should have been inlined before this pass")
            },
            Node::Evaluator(_e) => {
                unreachable!("Evaluators should have been inlined before this pass")
            },
            Node::Enf(e) => to_link_and(e.clone(), graph, |g, el| self.visit_enf_bis(g, el)),
            Node::Boundary(b) => {
                to_link_and(b.clone(), graph, |g, el| self.visit_boundary_bis(g, el))
            },
            Node::Add(a) => to_link_and(a.clone(), graph, |g, el| self.visit_add_bis(g, el)),
            Node::Sub(s) => to_link_and(s.clone(), graph, |g, el| self.visit_sub_bis(g, el)),
            Node::Mul(m) => to_link_and(m.clone(), graph, |g, el| self.visit_mul_bis(g, el)),
            Node::Exp(e) => to_link_and(e.clone(), graph, |g, el| self.visit_exp_bis(g, el)),
            Node::If(i) => to_link_and(i.clone(), graph, |g, el| self.visit_if_bis(g, el)),
            Node::For(f) => to_link_and(f.clone(), graph, |g, el| self.visit_for_bis(g, el)),
            Node::Call(c) => to_link_and(c.clone(), graph, |g, el| self.visit_call_bis(g, el)),
            Node::Fold(f) => to_link_and(f.clone(), graph, |g, el| self.visit_fold_bis(g, el)),
            Node::Vector(v) => to_link_and(v.clone(), graph, |g, el| self.visit_vector_bis(g, el)),
            Node::Matrix(m) => to_link_and(m.clone(), graph, |g, el| self.visit_matrix_bis(g, el)),
            Node::Accessor(a) => {
                to_link_and(a.clone(), graph, |g, el| self.visit_accessor_bis(g, el))
            },
            Node::BusOp(_b) => Ok(None),
            Node::Parameter(p) => {
                to_link_and(p.clone(), graph, |g, el| self.visit_parameter_bis(g, el))
            },
            Node::Value(v) => to_link_and(v.clone(), graph, |g, el| self.visit_value_bis(g, el)),
            Node::None(_) => Ok(None),
        };

        // We update the node if needed
        if let Some(updated_op) = updated_op? {
            node.as_op().unwrap().set(&updated_op);
        }

        Ok(())
    }
}

impl Visitor for UnrollingSecondPass<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    // The root nodes visited during the second pass are the children of For nodes to inline
    fn root_nodes_to_visit(&self, _graph: &Graph) -> Vec<Link<Node>> {
        self.bodies_to_inline
            .iter()
            .map(|(k, _v)| k)
            .cloned()
            .map(|op| op.as_node())
            .collect::<Vec<_>>()
    }
    fn run(&mut self, graph: &mut Graph) -> Result<(), CompileError> {
        for root in self.root_nodes_to_visit(graph).iter() {
            // Set context to inline the body for this index
            let for_inlining_context = self.bodies_to_inline.iter().find_map(|(node, context)| {
                if Rc::ptr_eq(&node.clone().as_node().link, &root.link) {
                    Some(context.clone())
                } else {
                    None
                }
            });

            self.for_inlining_context = for_inlining_context;
            // We inline a new body, so we clear the nodes to replace and the parameters for the ref
            // node
            self.nodes_to_replace.clear();
            self.params_for_ref_node.clear();

            self.scan_node(graph, self.for_inlining_context.clone().unwrap().body.as_node())?;
            while let Some(node) = self.work_stack().pop() {
                self.visit_node(graph, node.clone())?;
            }

            // We have finished inlining the body, we can now replace the Root node with the body
            let body = self.for_inlining_context.clone().unwrap().body;
            let new_node = self.nodes_to_replace.get(&body.get_ptr()).unwrap().1.clone();

            // If there is a selector, we need to enforce it on the body
            let new_node_with_selector_if_needed =
                if let Some(selector) = self.for_inlining_context.clone().unwrap().selector {
                    if let Op::Vector(new_node_vector) = new_node.borrow().deref() {
                        let new_node_vec = new_node_vector.children().borrow().deref().clone();
                        let mut new_vec = vec![];
                        for new_node_child in new_node_vec.into_iter() {
                            let new_node_child_with_selector = Mul::create(
                                duplicate_node(selector.clone(), &mut HashMap::new()),
                                new_node_child,
                                root.span(),
                            );
                            new_vec.push(new_node_child_with_selector);
                        }
                        Vector::create(new_vec, root.span())
                    } else {
                        Mul::create(selector, new_node, root.span())
                    }
                } else {
                    new_node
                };

            root.as_op().unwrap().set(&new_node_with_selector_if_needed);

            // Reset context to None
            self.for_inlining_context = None;
        }

        Ok(())
    }
    fn visit_node(&mut self, _graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        if node.is_stale() {
            return Ok(());
        }
        if let Some(op) = node.clone().as_op() {
            duplicate_node_or_replace(
                &mut self.nodes_to_replace,
                op,
                self.for_inlining_context.clone().unwrap().iterators.clone(),
                self.for_inlining_context.clone().unwrap().ref_node.as_node(),
                Some(
                    self.all_for_nodes
                        .get(&self.for_inlining_context.clone().unwrap().ref_node.get_ptr())
                        .unwrap()
                        .1
                        .clone(),
                ),
                &mut self.params_for_ref_node,
            );
        } else {
            unreachable!("UnrollingSecondPass::visit_node on a non-Op node: {:?}", node);
        }
        Ok(())
    }
}

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

/// Helper function used to check whether two sets of constraints have disjoint conditions.
/// This is used to combine constraints in the `visit_if_bis` method, as we can only combine
/// constraints that have disjoint selectors.
fn have_disjoint_conditions(
    cur_constraints: Vec<Vec<(Link<Op>, Link<Op>)>>,
    new_constraints: &[(Link<Op>, Link<Op>)],
) -> bool {
    for (condition, _) in new_constraints {
        if cur_constraints.iter().flatten().any(|(c, _)| c == condition) {
            return false; // Duplicate condition found
        }
    }
    true
}
