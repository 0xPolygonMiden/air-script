use std::{collections::HashMap, ops::Deref};

use air_parser::ast::AccessType;
use miden_diagnostics::{DiagnosticsHandler, Spanned};

use crate::{
    CompileError,
    ir::{
        Accessor, Graph, Link, MirType, MirValue, Node, Op, Owner, Parameter, Parent,
        SpannedMirValue, TraceAccess, Value, Vector,
    },
    passes::{
        Visitor,
        unrolling::{
            ForInliningContext, visit_add_bis, visit_boundary_bis, visit_enf_bis, visit_exp_bis,
            visit_fold_bis, visit_mul_bis, visit_sub_bis, visit_value_bis, visit_vector_bis,
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

/// Unrolls an `Accessor` with `AccessType::Default` access type.
fn unroll_accessor_default_access_type(
    indexable: Link<Op>,
    accessor_offset: usize,
) -> Option<Link<Op>> {
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
            return Some(new_node);
        }
    }
    Some(indexable.clone())
}

/// Unrolls an `Accessor` with `AccessType::Index` access type.
fn unroll_accessor_index_access_type(
    indexable: Link<Op>,
    index: usize,
    accessor_offset: usize,
) -> Option<Link<Op>> {
    // Check that the child node is a vector, raise diag otherwise
    // Replace the current node by the index-th element of the vector
    // Raise diag if index is out of bounds
    if let Op::Vector(indexable_vector) = indexable.borrow().deref() {
        let indexable_vec = indexable_vector.children().borrow().clone();
        let child_accessed = indexable_vec
            .get(index)
            .unwrap_or_else(|| panic!("Index access out of bounds for indexable: {indexable:?}"));
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
                    Some(new_node)
                },
                _ => Some(child_accessed.clone()),
            }
        } else {
            Some(child_accessed.clone())
        }
    } else {
        unreachable!("indexable is {:?}", indexable); // raise diag
    }
}

/// Unrolls an `Accessor` with `AccessType::Matrix` access type.
fn unroll_accessor_matrix_access_type(
    indexable: Link<Op>,
    row: usize,
    col: usize,
) -> Option<Link<Op>> {
    // Check that the child node is a matrix, raise diag otherwise
    // Replace the current node by the index-th element of the vector
    // Raise diag if index is out of bounds
    if let Op::Vector(indexable_vector) = indexable.borrow().deref() {
        let indexable_vec = indexable_vector.children().borrow().clone();
        let row_accessed = indexable_vec
            .get(row)
            .unwrap_or_else(|| panic!("Matrix access out of bounds for indexable: {indexable:?}"));
        if let Op::Vector(row_accessed_vector) = row_accessed.borrow().deref() {
            let row_accessed_vec = row_accessed_vector.children().borrow().clone();
            let child_accessed = row_accessed_vec.get(col).unwrap_or_else(|| {
                panic!("Matrix access out of bounds for indexable: {indexable:?}")
            });
            Some(child_accessed.clone())
        } else {
            unreachable!("unexpected non-vector child of a Matrix: {:?}", row_accessed);
        }
    } else if let Op::Matrix(indexable_matrix) = indexable.borrow().deref() {
        let indexable_vec = indexable_matrix.children().borrow().clone();
        let row_accessed = indexable_vec
            .get(row)
            .unwrap_or_else(|| panic!("Matrix access out of bounds for indexable: {indexable:?}"));
        if let Op::Vector(row_accessed_vector) = row_accessed.borrow().deref() {
            let row_accessed_vec = row_accessed_vector.children().borrow().clone();
            let child_accessed = row_accessed_vec.get(col).unwrap_or_else(|| {
                panic!("Matrix access out of bounds for indexable: {indexable:?}")
            });
            Some(child_accessed.clone())
        } else {
            unreachable!("unexpected non-vector child of a Matrix: {:?}", row_accessed);
        }
    } else {
        unreachable!("unexpected matrix access type on indexable {:?}", indexable);
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
            Node::Boundary(b) => b.to_link().map_or(Ok(None), visit_boundary_bis)?,
            Node::Add(a) => a.to_link().map_or(Ok(None), visit_add_bis)?,
            Node::Sub(s) => s.to_link().map_or(Ok(None), visit_sub_bis)?,
            Node::Mul(m) => m.to_link().map_or(Ok(None), visit_mul_bis)?,
            Node::Exp(e) => e.to_link().map_or(Ok(None), visit_exp_bis)?,
            Node::Fold(f) => f.to_link().map_or(Ok(None), visit_fold_bis)?,
            Node::Vector(v) => v.to_link().map_or(Ok(None), visit_vector_bis)?,
            Node::Accessor(a) => a.to_link().map_or(Ok(None), visit_accessor_bis)?,
            Node::Value(v) => v.to_link().map_or(Ok(None), visit_value_bis)?,
            Node::For(f) => f.to_link().map_or(Ok(None), |el| self.visit_for_bis(el))?,
            Node::Parameter(p) => {
                p.to_link().map_or(Ok(None), |el| self.visit_parameter_bis(el))?
            },
            Node::BusOp(_b) => None,
            Node::Matrix(_) => None, // Matrix are already unrolled, we have nothing to do
            Node::If(_i) => None,
            Node::None(_) => None,
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

/// Unrolls an `Accessor` depending on its `AccessType`.
///
/// Note: If the `indexable` is a `Parameter` (referencing a `For` node), we do not unroll the
/// `Accessor`, in order to handle nested `For` nodes.
pub fn visit_accessor_bis(accessor: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    let accessor_ref = accessor.as_accessor().unwrap();
    let indexable = accessor_ref.indexable.clone();
    let access_type = accessor_ref.access_type.clone();
    let offset = accessor_ref.offset;
    // If the indexable is a parameter, we keep the accessor as is, in
    // order to handle nested `For` nodes
    if indexable.clone().as_parameter().is_none() {
        match access_type {
            AccessType::Default => {
                return Ok(unroll_accessor_default_access_type(indexable, offset));
            },
            AccessType::Index(index) => {
                return Ok(unroll_accessor_index_access_type(indexable, index, offset));
            },
            AccessType::Matrix(row, col) => {
                return Ok(unroll_accessor_matrix_access_type(indexable, row, col));
            },
            AccessType::Slice(_range_expr) => {
                unreachable!(); // Slices are not scalar, raise diag
            },
        }
    }
    Ok(None)
}

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
