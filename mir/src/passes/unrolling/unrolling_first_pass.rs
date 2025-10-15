use std::{collections::HashMap, ops::Deref};

use miden_diagnostics::{DiagnosticsHandler, Spanned};

use crate::{
    CompileError,
    ir::{
        Accessor, ConstantValue, Graph, Link, MirAccessType, MirType, MirValue, Node, Op, Owner,
        Parameter, Parent, SpannedMirValue, Value, Vector,
    },
    passes::{
        Visitor, handle_accessor_visit,
        unrolling::{
            ForInliningContext, visit_enf_bis, visit_fold_bis, visit_value_bis, visit_vector_bis,
        },
    },
};

pub struct UnrollingFirstPass<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,
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
            bodies_to_inline: vec![],
            params_for_ref_node: HashMap::new(),
            all_for_nodes: HashMap::new(),
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

        let owner_ref =
            parameter.as_parameter().unwrap().ref_node.to_link().expect("Invalid Ref node");

        self.params_for_ref_node
            .entry(owner_ref.get_ptr())
            .or_default()
            .push(parameter.clone());
        Ok(None)
    }

    fn visit_accessor_bis(&mut self, accessor: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
        let accessor_ref = accessor.as_accessor().unwrap();
        let indexable = accessor_ref.indexable.clone();
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
        // We keep a reference to all `For` nodes to avoid dropping the backlinks stored in
        // `Parameters`
        if let Some(owner) = node.clone().as_owner()
            && let Some(op) = owner.clone().as_op()
            && let Some(_for_node) = op.as_for()
        {
            self.all_for_nodes.insert(op.get_ptr(), (op.clone(), owner.clone()));
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
