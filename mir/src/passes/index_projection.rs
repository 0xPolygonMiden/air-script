//! Index projection pass for MIR.
//!
//! The aim is to avoid eager vector/matrix expansion when only one element is used. We do this
//! by recognizing pure function returns that are immediately indexed and replacing the accessor
//! with the projected element. The tradeoff is a conservative check (purity + constant indices),
//! so we intentionally skip some projection opportunities.

use std::{collections::HashMap, ops::Deref};

use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;

use super::{duplicate_node_or_replace_with_interner, get_inner_const, visitor::Visitor};
use crate::{
    CompileError,
    ir::{Graph, Link, Mir, MirAccessType, Node, Op, OpInterner, OwnerId, Parent},
};

/// Inline projections of vector/matrix-returning pure functions.
///
/// This avoids eagerly expanding large vectors when the call result is immediately indexed.
pub struct IndexProjection<'a> {
    _diagnostics: &'a DiagnosticsHandler,
    work_stack: Vec<Link<Node>>,
    seen_accessors: HashMap<usize, ()>,
    op_interner: Option<OpInterner>,
}

impl<'a> IndexProjection<'a> {
    /// Create a new index projection pass.
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self {
            _diagnostics: diagnostics,
            work_stack: Vec::new(),
            seen_accessors: HashMap::new(),
            op_interner: None,
        }
    }

    fn project_return_expr(
        &self,
        return_expr: &Link<Op>,
        access_type: &MirAccessType,
    ) -> Option<Link<Op>> {
        match access_type {
            MirAccessType::Index(index) => {
                let idx = get_inner_const(index)? as usize;
                let expr = return_expr.clone();
                if let Some(vector) = expr.as_vector() {
                    let children_link = vector.children();
                    let children = children_link.borrow();
                    children.get(idx).cloned()
                } else {
                    None
                }
            },
            MirAccessType::Matrix(row, col) => {
                let row_idx = get_inner_const(row)? as usize;
                let col_idx = get_inner_const(col)? as usize;
                let expr = return_expr.clone();
                if let Some(matrix) = expr.as_matrix() {
                    let rows_link = matrix.children();
                    let rows = rows_link.borrow();
                    let row = rows.get(row_idx)?.clone();
                    let row_vec = row.as_vector()?;
                    let cols_link = row_vec.children();
                    let cols = cols_link.borrow();
                    cols.get(col_idx).cloned()
                } else {
                    None
                }
            },
            MirAccessType::Default => None,
        }
    }

    fn project_accessor(&mut self, accessor: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
        if self.seen_accessors.contains_key(&accessor.get_ptr()) {
            return Ok(None);
        }
        self.seen_accessors.insert(accessor.get_ptr(), ());

        let (access_type, indexable, offset) = {
            let Some(accessor_ref) = accessor.as_accessor() else {
                return Ok(None);
            };
            (
                accessor_ref.access_type.clone(),
                accessor_ref.indexable.clone(),
                accessor_ref.offset,
            )
        };

        if offset != 0 {
            return Ok(None);
        }

        let Some(call) = indexable.as_call() else {
            return Ok(None);
        };

        let callee = call.function.clone();
        let return_expr = {
            let Some(func) = callee.as_function() else {
                return Ok(None);
            };
            match func.body.borrow().last() {
                Some(expr) => expr.clone(),
                None => return Ok(None),
            }
        };

        let Some(projected) = self.project_return_expr(&return_expr, &access_type) else {
            return Ok(None);
        };

        let args = call.arguments.borrow().clone();
        let ref_owner_id = callee.as_owner().owner_id();

        let mut nodes_to_replace: HashMap<usize, (Link<Op>, Link<Op>)> = HashMap::new();
        let mut params_for_ref_node: HashMap<OwnerId, Vec<Link<Op>>> = HashMap::new();
        let mut owner_id_map: HashMap<OwnerId, OwnerId> = HashMap::new();
        let mut param_cache: HashMap<(OwnerId, usize, bool), Link<Op>> = HashMap::new();

        if self.op_interner.is_none() {
            self.op_interner = Some(OpInterner::new());
        }

        duplicate_node_or_replace_with_interner(
            &mut nodes_to_replace,
            projected.clone(),
            args,
            ref_owner_id,
            &mut params_for_ref_node,
            &mut owner_id_map,
            &mut param_cache,
            &mut self.op_interner,
        );

        let new_node =
            nodes_to_replace.get(&projected.get_ptr()).map(|(_, new_node)| new_node.clone());

        Ok(new_node)
    }
}

impl Pass for IndexProjection<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut input: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        if std::env::var("AIR_DISABLE_INDEX_PROJECTION").is_ok() {
            return Ok(input);
        }
        Visitor::run(self, input.constraint_graph_mut())?;
        Ok(input)
    }
}

impl Visitor for IndexProjection<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }

    fn root_nodes_to_visit(&self, graph: &Graph) -> Vec<Link<Node>> {
        let functions = graph.get_function_nodes();
        let evaluators = graph.get_evaluator_nodes();
        let boundary_constraints_roots_ref = graph.boundary_constraints_roots.borrow();
        let integrity_constraints_roots_ref = graph.integrity_constraints_roots.borrow();
        let bus_roots: Vec<_> = graph
            .buses
            .values()
            .flat_map(|b| b.borrow().clone().columns.into_iter().collect::<Vec<_>>())
            .collect();

        boundary_constraints_roots_ref
            .clone()
            .into_iter()
            .map(|bc| bc.as_node())
            .chain(integrity_constraints_roots_ref.clone().into_iter().map(|ic| ic.as_node()))
            .chain(bus_roots.into_iter().map(|b| b.as_node()))
            .chain(evaluators.into_iter().map(|e| e.as_node()))
            .chain(functions.into_iter().map(|f| f.as_node()))
            .collect()
    }

    fn visit_node(&mut self, _graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        if node.is_stale() {
            return Ok(());
        }

        let updated_op = match node.borrow().deref() {
            Node::Accessor(accessor) => {
                accessor.to_link().map_or(Ok(None), |el| self.project_accessor(el))?
            },
            _ => None,
        };

        if let Some(updated) = updated_op {
            node.as_op().unwrap().set(&updated);
        }

        Ok(())
    }
}
