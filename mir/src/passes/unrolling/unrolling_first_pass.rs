use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
};

use air_parser::ast::AccessType;
use miden_diagnostics::{DiagnosticsHandler, SourceSpan, Spanned};

use crate::{
    CompileError,
    ir::{
        Accessor, Add, BackLink, Boundary, ConstantValue, Enf, Exp, FoldOperator, Graph, Link,
        Matrix, MirType, MirValue, Mul, Node, Op, Owner, Parameter, Parent, SpannedMirValue, Sub,
        TraceAccess, TraceAccessBinding, Value, Vector,
    },
    passes::{
        Visitor,
        unrolling::{ForInliningContext, match_optimizer::MatchOptimizer},
    },
};

pub struct UnrollingFirstPass<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,
    // current evaluations of nodes at random points
    match_optimizer: MatchOptimizer,
    // For each child of a For node encountered, we store the context to inline it in the second
    // pass
    pub bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,
    // We keep track of all parameters referencing a given For node
    params_for_ref_node: HashMap<usize, Vec<Link<Op>>>,
    // We keep a reference to For nodes in order to avoid the backlinks stored in Parameters
    // referencing them to be dropped
    pub all_for_nodes: HashMap<usize, (Link<Op>, Link<Owner>)>,
}

impl<'a> UnrollingFirstPass<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self {
            diagnostics,
            work_stack: vec![],
            match_optimizer: MatchOptimizer::default(),
            bodies_to_inline: vec![],
            params_for_ref_node: HashMap::new(),
            all_for_nodes: HashMap::new(),
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
    let mut updated_binary_op = None;

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
            updated_binary_op = Some(Vector::create(new_vec, parent.span()));
        }
    }

    Ok(updated_binary_op)
}

fn unroll_accessor_default_access_type(
    indexable: Link<Op>,
    accessor_offset: usize,
) -> Option<Link<Op>> {
    let mut updated_accessor = Some(indexable.clone());

    if let Some(value) = indexable.clone().as_value() {
        let mir_value = value.value.value.clone();

        if let MirValue::TraceAccess(trace_access) = mir_value {
            let new_node = Value::create(SpannedMirValue {
                span: value.value.span(),
                value: MirValue::TraceAccess(TraceAccess {
                    segment: trace_access.segment,
                    column: trace_access.column,
                    row_offset: trace_access.row_offset + accessor_offset,
                }),
            });
            updated_accessor = Some(new_node);
        }
    }
    updated_accessor
}

fn unroll_accessor_index_access_type(
    indexable: Link<Op>,
    index: usize,
    accessor_offset: usize,
) -> Option<Link<Op>> {
    let updated_accessor;

    // Check that the child node is a vector, raise diag otherwise
    // Replace the current node by the index-th element of the vector
    // Raise diag if index is out of bounds
    if let Op::Vector(indexable_vector) = indexable.borrow().deref() {
        let indexable_vec = indexable_vector.children().borrow().deref().clone();
        let child_accessed = match indexable_vec.get(index) {
            Some(child_accessed) => child_accessed,
            None => unreachable!(), // raise diag
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
                            row_offset: trace_access.row_offset + accessor_offset,
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
    } else {
        unreachable!("indexable is {:?}", indexable); // raise diag
    };
    updated_accessor
}

fn unroll_accessor_matrix_access_type(
    indexable: Link<Op>,
    row: usize,
    col: usize,
) -> Option<Link<Op>> {
    // Check that the child node is a matrix, raise diag otherwise
    // Replace the current node by the index-th element of the vector
    // Raise diag if index is out of bounds
    if let Op::Vector(indexable_vector) = indexable.borrow().deref() {
        let indexable_vec = indexable_vector.children().borrow().deref().clone();
        let row_accessed = match indexable_vec.get(row) {
            Some(row_accessed) => row_accessed,
            None => unreachable!("Matrix access out of bounds for indexable: {:?}", indexable),
        };
        if let Op::Vector(row_accessed_vector) = row_accessed.borrow().deref() {
            let row_accessed_vec = row_accessed_vector.children().borrow().deref().clone();
            let child_accessed = match row_accessed_vec.get(col) {
                Some(child_accessed) => child_accessed,
                None => unreachable!("Matrix access out of bounds for indexable: {:?}", indexable),
            };
            Some(child_accessed.clone())
        } else {
            unreachable!("unexpected non-vector child of a Matrix: {:?}", row_accessed);
        }
    } else if let Op::Matrix(indexable_matrix) = indexable.borrow().deref() {
        let indexable_vec = indexable_matrix.children().borrow().deref().clone();
        let row_accessed = match indexable_vec.get(row) {
            Some(row_accessed) => row_accessed,
            None => unreachable!("Matrix access out of bounds for indexable: {:?}", indexable),
        };
        if let Op::Vector(row_accessed_vector) = row_accessed.borrow().deref() {
            let row_accessed_vec = row_accessed_vector.children().borrow().deref().clone();
            let child_accessed = match row_accessed_vec.get(col) {
                Some(child_accessed) => child_accessed,
                None => unreachable!("Matrix access out of bounds for indexable: {:?}", indexable),
            };
            Some(child_accessed.clone())
        } else {
            unreachable!("unexpected non-vector child of a Matrix: {:?}", row_accessed);
        }
    } else {
        unreachable!("unexpected matrix access type on indexable {:?}", indexable);
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
            match &mir_value {
                MirValue::Constant(c) => match c {
                    ConstantValue::Felt(_) => {},
                    ConstantValue::Vector(v) => {
                        updated_value = Some(unroll_constant_vector(v, value_ref.span()));
                    },
                    ConstantValue::Matrix(m) => {
                        updated_value = Some(unroll_constant_matrix(m, value_ref.span()));
                    },
                },
                MirValue::TraceAccessBinding(trace_access_binding) => {
                    updated_value =
                        Some(unroll_trace_access_binding(trace_access_binding, value_ref.span()));
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
        }

        Ok(updated_value)
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
            let access_type = accessor_ref.access_type.clone();
            let offset = accessor_ref.offset;

            // If the indexable is a parameter, we keep the accessor as is, in
            // order to handle nested For nodes
            if indexable.clone().as_parameter().is_none() {
                match access_type {
                    AccessType::Default => {
                        updated_accessor = unroll_accessor_default_access_type(indexable, offset);
                    },
                    AccessType::Index(index) => {
                        updated_accessor =
                            unroll_accessor_index_access_type(indexable, index, offset);
                    },
                    AccessType::Matrix(row, col) => {
                        updated_accessor = unroll_accessor_matrix_access_type(indexable, row, col);
                    },
                    AccessType::Slice(_range_expr) => {
                        unreachable!(); // Slices are not scalar, raise diag
                    },
                }
            }
        }

        Ok(updated_accessor)
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
                unreachable!("Expected vector iterator in fold, found: {:?}", iterator_ref);
            };
            let iterator_nodes = iterator_vector.children().borrow().deref().clone();

            let mut acc_node = initial_value;
            for iterator_node in iterator_nodes {
                let new_acc_node = match operator {
                    FoldOperator::Add => Add::create(acc_node, iterator_node, fold_ref.span()),
                    FoldOperator::Mul => Mul::create(acc_node, iterator_node, fold_ref.span()),
                    FoldOperator::None => {
                        unreachable!("Unexpected unrolling of Fold with None FoldOperator")
                    },
                };
                acc_node = new_acc_node;
            }
            updated_fold = Some(acc_node);
        }

        Ok(updated_fold)
    }

    /// Visiting an If node consists into evaluating all the main trace constraints contained in the
    /// match arms, and combining them to optimize the resulting vector of constraints if
    /// possible. We handle bus related constraints separately, as they cannot be combined with
    /// main trace constraints.
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

        self.match_optimizer.evaluate_match_arms(
            &mut node_evals,
            &mut constraints_evaluation_indices,
            &mut bus_related_constraints,
            match_arms,
        )?;

        // 2. For each evaluation, we keep track of the length of the vector of unique constraints
        //    evaluating to it
        // This len will be used to combine the constraints by taking the most constrained
        // evaluations first, in order to minimize the number of constraints in the
        // final vector
        let mut eval_lens = self.match_optimizer.compute_eval_lens(&constraints_evaluation_indices);

        // 3. Construct the new vector of combined constraints
        let mut new_vec = self.match_optimizer.reduce_main_constraints(
            &mut constraints_evaluation_indices,
            &mut eval_lens,
            if_ref.span,
        );

        // 4. Add all the constraints that are bus-related
        self.match_optimizer.add_bus_constaints(
            &mut bus_related_constraints,
            &mut new_vec,
            if_ref.span,
        );

        Ok(Some(Vector::create(new_vec, if_ref.span())))
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
            // If there is a selector, we need to enforce the selector on the body

            let for_node_clone = for_node.clone();
            let for_ref = for_node_clone.as_for().unwrap();
            let iterators_ref = for_ref.iterators.borrow();
            let iterators = iterators_ref.deref();
            let expr = for_ref.expr.clone();
            let selector = for_ref.selector.clone();

            let iterator_expected_len = validate_iterators_and_get_expected_len(iterators);

            let mut new_vec = vec![];
            for i in 0..iterator_expected_len {
                let new_node =
                    Parameter::create(i, MirType::Felt, for_node.as_for().unwrap().deref().span());
                new_vec.push(new_node.clone());

                let iterators_i = iterators
                    .iter()
                    .map(|iterator| get_iterator_child(iterator.clone(), i))
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
    }

    fn visit_matrix_bis(
        &mut self,
        _graph: &mut Graph,
        _matrix: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        Ok(None) // Matrix are already unrolled, we have nothing to do
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
            Node::For(f) => to_link_and(f.clone(), graph, |g, el| self.visit_for_bis(g, el)),
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
            Node::Function(_) | Node::Evaluator(_) | Node::Call(_) => {
                unreachable!(
                    "Unexpected node during Unrolling: Function, Evaluators and Calls should have been inlined before this pass. Found: {:?}",
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

/// Sanity check that all iterators have the same length.
/// Note that semantic analysis should have already checked they are valid.
fn validate_iterators_and_get_expected_len(iterators: &[Link<Op>]) -> usize {
    if iterators.is_empty() {
        unreachable!("Semantic analysis should have catched empty iterators");
    }
    let iterator_expected_len = compute_iterator_len(iterators[0].clone());
    for iterator in iterators.iter().skip(1) {
        let iterator_len = compute_iterator_len(iterator.clone());
        if iterator_len != iterator_expected_len {
            unreachable!("Semantic analysis should have catched iterator length mismatch");
        }
    }
    iterator_expected_len
}

/// Computes the length of a node that is used as an iterator in a For node.
fn compute_iterator_len(iterator: Link<Op>) -> usize {
    match iterator.borrow().deref() {
        Op::Vector(vector) => vector.size,
        Op::Matrix(matrix) => matrix.size,
        Op::Accessor(accessor) => match &accessor.access_type {
            AccessType::Default => compute_iterator_len(accessor.indexable.clone()),
            AccessType::Slice(range_expr) => range_expr.to_slice_range().count(),
            AccessType::Index(_) => match accessor.indexable.borrow().deref() {
                Op::Vector(_) => 1,
                Op::Matrix(matrix) => {
                    let children = matrix.children().borrow().deref().clone();
                    match children.first() {
                        Some(first_row) => match first_row.as_vector() {
                            Some(row_vector) => row_vector.size,
                            _ => unreachable!("Unexpected row type in matrix"),
                        },
                        None => {
                            unreachable!("Unexpected empty matrix");
                        },
                    }
                },
                _ => unreachable!("Unexpected index into non indexable type"),
            },
            AccessType::Matrix(..) => 1,
        },
        Op::Parameter(parameter) => match parameter.ty {
            MirType::Felt => 1,
            MirType::Vector(l) => l,
            MirType::Matrix(l, _) => l,
        },
        _ => 1,
    }
}

/// Returns the i-th child of an iterator node.
fn get_iterator_child(op: Link<Op>, i: usize) -> Link<Op> {
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
                Op::Parameter(_parameter) => Accessor::create(
                    accessor.indexable.clone(),
                    AccessType::Index(i),
                    0,
                    accessor.span(),
                ),
                _ => op.clone(),
            }
        },
        _ => op.clone(),
    }
}
