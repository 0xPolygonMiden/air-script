//! Unrolling first pass.
//!
//! Traverses the graph, unrolling most nodes and collecting `For` contexts
//! for the second pass.

use std::{collections::HashMap, ops::Deref};

use miden_diagnostics::{DiagnosticsHandler, Spanned};

use crate::{
    CompileError,
    ir::{
        Accessor, ConstantValue, Graph, Link, MirAccessType, MirType, MirValue, Node, Op, OwnerId,
        Parameter, Parent, SpannedMirValue, Value, Vector,
    },
    passes::{
        Visitor, handle_accessor_visit, should_skip_accessor_unroll,
        unrolling::{
            ForInliningContext, visit_enf_bis, visit_fold_bis, visit_value_bis, visit_vector_bis,
        },
    },
};

/// First pass of unrolling: records `For` contexts and rewrites other nodes.
pub struct UnrollingFirstPass<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,
    trace_progress: bool,
    progress_every: usize,
    pub(super) nodes_visited: usize,
    // For each child of a For node encountered, we store the context to inline it in the second
    // pass
    pub bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,
    // We keep track of all parameters referencing a given For node
    params_for_ref_node: HashMap<OwnerId, Vec<Link<Op>>>,
}

impl<'a> UnrollingFirstPass<'a> {
    /// Construct a new first-pass unroller.
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        // AIR_UNROLL_PROGRESS/AIR_UNROLL_PROGRESS_EVERY emit periodic progress for large graphs.
        let trace_progress = std::env::var("AIR_UNROLL_PROGRESS").is_ok();
        let progress_every = std::env::var("AIR_UNROLL_PROGRESS_EVERY")
            .ok()
            .and_then(|val| val.parse::<usize>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(100_000);
        Self {
            diagnostics,
            work_stack: vec![],
            trace_progress,
            progress_every,
            nodes_visited: 0,
            bodies_to_inline: vec![],
            params_for_ref_node: HashMap::new(),
        }
    }
}

// For the first pass of Unrolling, we use a tweaked version of the Visitor trait,
// each visit_*_bis function returns an `Option<Link<Op>>` instead of `Result<(), CompileError>`,
// to mutate the nodes (e.g. modifying an `Operation<Vectors>` to `Vector<Operations>`)
impl UnrollingFirstPass<'_> {
    fn visit_parameter_bis(
        &mut self,
        parameter: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // FIXME: Just check that the parameter is a scalar, raise diag otherwise
        // List comprehension bodies should only be scalar expressions

        let param_ref = parameter.as_parameter().unwrap();
        if param_ref.owner_id.is_unknown() {
            eprintln!("unrolling_first_pass: invalid owner_id for parameter: {:?}", param_ref);
            return Err(CompileError::Failed);
        }

        self.params_for_ref_node
            .entry(param_ref.owner_id)
            .or_default()
            .push(parameter.clone());
        Ok(None)
    }

    fn visit_accessor_bis(&mut self, accessor: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
        let Some(accessor_ref) = accessor.as_accessor() else {
            // This node may have been rewritten already; skip stale accessors.
            return Ok(None);
        };
        let indexable = accessor_ref.indexable.clone();
        if should_skip_accessor_unroll(&indexable) {
            return Ok(None);
        }
        if indexable.clone().as_parameter().is_none() {
            handle_accessor_visit(accessor.clone(), false, self.diagnostics)
        } else {
            // We keep accessors wrapping parameters to allow for nested list comprehensions.
            Ok(None)
        }
    }

    fn visit_for_bis(&mut self, for_node: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
        // For each value produced by the iterators, we need to:
        // - Duplicate the body
        // - Visit the body and replace the Variables with the value (with the correct index
        //   depending on the binding)
        // If there is a selector, we need to enforce the selector on the body

        let (iterators, expr, selector, for_span) = {
            let for_ref = for_node.as_for().unwrap();
            let iterators = for_ref.iterators.borrow().clone();
            let expr = for_ref.expr.clone();
            let selector = for_ref.selector.clone();
            let span = for_ref.span();
            (iterators, expr, selector, span)
        };

        let ref_owner_id = for_node.as_owner().unwrap().owner_id();
        let iterator_expected_len = validate_iterators_and_get_expected_len(&iterators);
        // AIR_DEBUG_FOR_ITER dumps iterator shapes for For nodes during unrolling.
        if std::env::var("AIR_DEBUG_FOR_ITER").is_ok() {
            let iter_debug = iterators.iter().map(|it| it.debug()).collect::<Vec<_>>().join(", ");
            eprintln!(
                "unrolling_first_pass: for owner_id={:?} len={} iterators=[{}]",
                ref_owner_id, iterator_expected_len, iter_debug
            );
        }

        let mut new_vec = vec![];

        for i in 0..iterator_expected_len {
            // Create a placeholder parameter for each iteration output. These placeholders are
            // later replaced with the inlined body for that iteration.
            let new_node = Parameter::create(i, MirType::Felt, for_span);
            if let Some(mut param) = new_node.as_parameter_mut() {
                // Mark as a For-output placeholder so nested unrolling can track contexts.
                param.set_for_output(true);
            }
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
                    ref_owner_id,
                },
            ));
            // AIR_DEBUG_UNROLL_CTX traces context creation for nested unrolling.
            if std::env::var("AIR_DEBUG_UNROLL_CTX").is_ok() {
                eprintln!(
                    "unrolling_first_pass: ctx owner_id={:?} body_ptr={}",
                    ref_owner_id,
                    expr.get_ptr()
                );
            }
        }

        let new_vec_op = Vector::create(new_vec.clone(), for_node.span());
        for param in new_vec {
            param.as_parameter_mut().unwrap().set_owner_id(ref_owner_id);
        }

        if let Some(params) = self.params_for_ref_node.get(&ref_owner_id).cloned() {
            for param in params.iter() {
                param.as_parameter_mut().unwrap().set_owner_id(ref_owner_id);
            }
        }
        Ok(Some(new_vec_op))
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

    fn visit_node(&mut self, _graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        if self.trace_progress {
            self.nodes_visited += 1;
            if self.nodes_visited % self.progress_every == 0 {
                eprintln!("mir: unrolling first pass visited {} nodes", self.nodes_visited);
            }
        }
        // In this pass, we both need to dispatch the visitor depending on the node type,
        // and also mutate the node if needed. We implement custom visit_*_bis methods
        // that returns a Some(updated_node) if we need to update the node's value.
        let updated_op: Option<Link<Op>> = match node.borrow().deref() {
            Node::Enf(e) => e.to_link().map_or(Ok(None), visit_enf_bis)?,
            Node::Fold(f) => f.to_link().map_or(Ok(None), visit_fold_bis)?,
            Node::Vector(v) => v.to_link().map_or(Ok(None), visit_vector_bis)?,
            Node::Accessor(a) => a.to_link().map_or(Ok(None), |el| self.visit_accessor_bis(el))?,
            Node::Value(v) => v.to_link().map_or(Ok(None), visit_value_bis)?,
            Node::For(f) => f.to_link().map_or(Ok(None), |el| self.visit_for_bis(el))?,
            Node::Parameter(p) => {
                p.to_link().map_or(Ok(None), |el| self.visit_parameter_bis(el))?
            },
            Node::Boundary(_)
            | Node::Add(_)
            | Node::Sub(_)
            | Node::Mul(_)
            | Node::Exp(_)
            | Node::BusOp(_)
            | Node::Matrix(_)
            | Node::If(_)
            | Node::None(_) => None,
            _ => {
                unreachable!(
                    "Unexpected node during Unrolling: Function, Evaluators and Calls should have been inlined before this pass. Found: {:?}",
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

/// Sanity check that all iterators have the same length.
/// Note that semantic analysis should have already checked they are valid.
fn validate_iterators_and_get_expected_len(iterators: &[Link<Op>]) -> usize {
    if iterators.is_empty() {
        unreachable!("Semantic analysis should have caught empty iterators");
    }
    let iterator_expected_len = compute_iterator_len(iterators[0].clone());
    for iterator in iterators.iter().skip(1) {
        let iterator_len = compute_iterator_len(iterator.clone());
        if iterator_len != iterator_expected_len {
            unreachable!("Semantic analysis should have caught iterator length mismatch");
        }
    }
    iterator_expected_len
}

/// Computes the length of a node that is used as an iterator in a `For` node.
fn compute_iterator_len(iterator: Link<Op>) -> usize {
    match iterator.borrow().deref() {
        Op::Vector(vector) => vector.size,
        Op::Matrix(matrix) => matrix.size,
        Op::Accessor(accessor) => match &accessor.access_type {
            MirAccessType::Default => compute_iterator_len(accessor.indexable.clone()),
            MirAccessType::Index(_) => match accessor.indexable.borrow().deref() {
                Op::Vector(_) => 1,
                Op::Matrix(matrix) => {
                    let children = matrix.children().borrow().clone();
                    children
                        .first()
                        .expect("Unexpected empty matrix")
                        .as_vector()
                        .expect("Expected vector for matrix row")
                        .size
                },
                _ => unreachable!("Unexpected index into non indexable type"),
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

/// Returns the i-th child of an iterator node.
fn get_iterator_child(op: Link<Op>, i: usize) -> Link<Op> {
    match op.borrow().deref() {
        Op::Vector(vector) => {
            let children = vector.children().borrow().clone();
            children[i].clone()
        },
        Op::Matrix(matrix) => {
            let children = matrix.children().borrow().clone();
            children[i].clone()
        },
        Op::Parameter(parameter) => {
            // If the iterator is a vector/matrix parameter, index into it for the i-th element.
            // If it's a scalar parameter, return it directly.
            match parameter.ty {
                MirType::Felt => op.clone(),
                MirType::Vector(_) | MirType::Matrix(..) => {
                    let mir_access_type = MirAccessType::Index(Value::create(SpannedMirValue {
                        span: parameter.span(),
                        value: MirValue::Constant(ConstantValue::Felt(i as u64)),
                    }));
                    Accessor::create(op.clone(), mir_access_type, 0, parameter.span())
                },
            }
        },
        Op::Accessor(accessor) => {
            match accessor.indexable.borrow().deref() {
                // If we access an outer loop parameter in the body of an inner
                // loop, we need to create
                // an Accessor for the correct index in this parameter
                Op::Parameter(_parameter) => {
                    let mir_access_type = MirAccessType::Index(Value::create(SpannedMirValue {
                        span: accessor.span(),
                        value: MirValue::Constant(ConstantValue::Felt(i as u64)),
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
}
