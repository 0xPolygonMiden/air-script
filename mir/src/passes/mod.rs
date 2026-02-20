//! MIR pass pipeline utilities and helpers.
//!
//! This module collects shared helpers used across the MIR pass pipeline. It exposes pass types
//! and common graph-duplication utilities so passes can safely clone subtrees while preserving
//! owner identity. The tradeoff is that deep duplication can grow the graph, so it is used
//! sparingly and usually paired with CSE/interning afterward.

mod constant_propagation;
mod cse;
mod index_projection;
mod inlining;
mod translate;
mod unrolling;
mod visitor;
use std::{collections::HashMap, ops::Deref};

pub use constant_propagation::ConstantPropagation;
pub use cse::Cse;
pub use index_projection::IndexProjection;
pub use inlining::Inlining;
use miden_diagnostics::{DiagnosticsHandler, Spanned};
pub use translate::AstToMir;
pub use unrolling::Unrolling;
pub use visitor::Visitor;

use crate::{
    CompileError,
    ir::{
        Accessor, Add, Boundary, BusOp, Call, Enf, Exp, Fold, For, If, Link, MatchArm, Matrix,
        MirAccessType, MirType, MirValue, Mul, Op, OpInterner, OwnerId, Parameter, Parent,
        PublicInputAccess, SpannedMirValue, Sub, TraceAccess, Value, Vector,
    },
};

/// Duplicate a MIR node and its children recursively.
///
/// Used when we need a deep clone of a subtree (e.g. when expanding let-bound values).
///
/// Note: `current_replace_map` is used to keep track of `For` nodes referenced by `Parameter`s
/// inside their bodies. When duplicating, those parameters must be re-bound to the new `For`.
pub fn duplicate_node(
    node: Link<Op>,
    current_replace_map: &mut HashMap<usize, (Link<Op>, Link<Op>)>,
) -> Link<Op> {
    let mut owner_id_map: HashMap<OwnerId, OwnerId> = HashMap::new();
    duplicate_node_with_owner_map(node, current_replace_map, &mut owner_id_map)
}

fn duplicate_node_with_owner_map(
    node: Link<Op>,
    current_replace_map: &mut HashMap<usize, (Link<Op>, Link<Op>)>,
    owner_id_map: &mut HashMap<OwnerId, OwnerId>,
) -> Link<Op> {
    match node.clone().borrow().deref() {
        Op::Enf(enf) => {
            let expr = enf.expr.clone();
            let new_expr = duplicate_node_with_owner_map(expr, current_replace_map, owner_id_map);
            Enf::create(new_expr, enf.span(), enf.tag.clone())
        },
        Op::Boundary(boundary) => {
            let expr = boundary.expr.clone();
            let kind = boundary.kind;
            let new_expr = duplicate_node_with_owner_map(expr, current_replace_map, owner_id_map);
            Boundary::create(new_expr, kind, boundary.span())
        },
        Op::Add(add) => {
            let lhs = add.lhs.clone();
            let rhs = add.rhs.clone();
            let new_lhs_node =
                duplicate_node_with_owner_map(lhs, current_replace_map, owner_id_map);
            let new_rhs_node =
                duplicate_node_with_owner_map(rhs, current_replace_map, owner_id_map);
            Add::create(new_lhs_node, new_rhs_node, add.span())
        },
        Op::Sub(sub) => {
            let lhs = sub.lhs.clone();
            let rhs = sub.rhs.clone();
            let new_lhs_node =
                duplicate_node_with_owner_map(lhs, current_replace_map, owner_id_map);
            let new_rhs_node =
                duplicate_node_with_owner_map(rhs, current_replace_map, owner_id_map);
            Sub::create(new_lhs_node, new_rhs_node, sub.span())
        },
        Op::Mul(mul) => {
            let lhs = mul.lhs.clone();
            let rhs = mul.rhs.clone();
            let new_lhs_node =
                duplicate_node_with_owner_map(lhs, current_replace_map, owner_id_map);
            let new_rhs_node =
                duplicate_node_with_owner_map(rhs, current_replace_map, owner_id_map);
            Mul::create(new_lhs_node, new_rhs_node, mul.span())
        },
        Op::Exp(exp) => {
            let lhs = exp.lhs.clone();
            let rhs = exp.rhs.clone();
            let new_lhs_node =
                duplicate_node_with_owner_map(lhs, current_replace_map, owner_id_map);
            let new_rhs_node =
                duplicate_node_with_owner_map(rhs, current_replace_map, owner_id_map);
            Exp::create(new_lhs_node, new_rhs_node, exp.span())
        },
        Op::If(if_node) => {
            let match_arms = if_node.match_arms.clone();
            let old_owner_id = if_node.owner_id;
            let new_owner_id = OwnerId::next();
            owner_id_map.insert(old_owner_id, new_owner_id);
            let new_match_arms = match_arms
                .borrow()
                .iter()
                .cloned()
                .map(|arm| {
                    let new_expr =
                        duplicate_node_with_owner_map(arm.expr, current_replace_map, owner_id_map);
                    let new_cond = duplicate_node_with_owner_map(
                        arm.condition,
                        current_replace_map,
                        owner_id_map,
                    );
                    MatchArm::new(new_expr, new_cond)
                })
                .collect::<Vec<_>>();
            let new_if = If::create(new_match_arms, if_node.span());
            if let Some(mut if_mut) = new_if.as_if_mut() {
                if_mut.owner_id = new_owner_id;
            }
            new_if
        },
        Op::For(for_node) => {
            let new_for_node: Link<Op> =
                For::create(Link::default(), Link::default(), Link::default(), for_node.span());
            let old_owner_id = for_node.owner_id;
            let new_owner_id = new_for_node.as_for().unwrap().owner_id;
            owner_id_map.insert(old_owner_id, new_owner_id);
            current_replace_map.insert(node.get_ptr(), (node, new_for_node.clone()));

            let iterators = for_node.iterators.clone();
            let body = for_node.expr.clone();
            let selector = for_node.selector.clone();
            let new_iterators = iterators
                .borrow()
                .iter()
                .cloned()
                .map(|x| duplicate_node_with_owner_map(x, current_replace_map, owner_id_map))
                .collect::<Vec<_>>();
            let new_selector =
                duplicate_node_with_owner_map(selector, current_replace_map, owner_id_map);
            let new_body = duplicate_node_with_owner_map(body, current_replace_map, owner_id_map);

            *new_for_node.as_for_mut().unwrap().iterators.borrow_mut() = new_iterators;
            *new_for_node.as_for_mut().unwrap().selector.borrow_mut() =
                new_selector.borrow().clone();
            *new_for_node.as_for_mut().unwrap().expr.borrow_mut() = new_body.borrow().clone();

            new_for_node
        },
        Op::Call(call) => {
            let arguments = call.arguments.clone();
            let function = call.function.clone();
            let new_arguments = arguments
                .borrow()
                .iter()
                .cloned()
                .map(|x| duplicate_node_with_owner_map(x, current_replace_map, owner_id_map))
                .collect::<Vec<_>>();
            Call::create(function, new_arguments, call.span())
        },
        Op::Fold(fold) => {
            let iterator = fold.iterator.clone();
            let operator = fold.operator.clone();
            let initial_value = fold.initial_value.clone();
            let new_iterator =
                duplicate_node_with_owner_map(iterator, current_replace_map, owner_id_map);
            let new_initial_value =
                duplicate_node_with_owner_map(initial_value, current_replace_map, owner_id_map);
            Fold::create(new_iterator, operator, new_initial_value, fold.span())
        },
        Op::Vector(vector) => {
            let children_link = vector.children().clone();
            let children_ref = children_link.borrow();
            let children = children_ref.deref();
            let new_children = children
                .iter()
                .cloned()
                .map(|x| duplicate_node_with_owner_map(x, current_replace_map, owner_id_map))
                .collect();
            Vector::create(new_children, vector.span())
        },
        Op::Matrix(matrix) => {
            let mut new_matrix = Vec::new();
            let children_link = matrix.children().clone();
            let children_ref = children_link.borrow();
            let children = children_ref.deref();
            for row in children.iter() {
                let row_children_link = row
                    .clone()
                    .as_vector()
                    .unwrap_or_else(|| panic!("expected Vector, found {row:?}"))
                    .children()
                    .clone();
                let row_children_ref = row_children_link.borrow();
                let row_children = row_children_ref.deref();
                let new_row_as_vec = row_children
                    .iter()
                    .cloned()
                    .map(|x| duplicate_node_with_owner_map(x, current_replace_map, owner_id_map))
                    .collect::<Vec<_>>();
                let new_row = Vector::create(new_row_as_vec, row.span());
                new_matrix.push(new_row);
            }
            Matrix::create(new_matrix, matrix.span())
        },
        Op::Accessor(accessor) => {
            let indexable = accessor.indexable.clone();
            let new_access_type =
                match accessor.access_type.clone() {
                    MirAccessType::Default => MirAccessType::Default,
                    MirAccessType::Index(index) => MirAccessType::Index(
                        duplicate_node_with_owner_map(index, current_replace_map, owner_id_map),
                    ),
                    MirAccessType::Matrix(row, col) => MirAccessType::Matrix(
                        duplicate_node_with_owner_map(row, current_replace_map, owner_id_map),
                        duplicate_node_with_owner_map(col, current_replace_map, owner_id_map),
                    ),
                };
            let offset = accessor.offset;
            let new_indexable =
                duplicate_node_with_owner_map(indexable, current_replace_map, owner_id_map);
            Accessor::create(new_indexable, new_access_type, offset, accessor.span())
        },
        Op::BusOp(bus_op) => {
            let bus = bus_op.bus.clone();
            let kind = bus_op.kind;
            let args: Vec<Link<Op>> = bus_op
                .args
                .iter()
                .map(|x| {
                    duplicate_node_with_owner_map(x.clone(), current_replace_map, owner_id_map)
                })
                .collect();
            BusOp::create(bus, kind, args, bus_op.span())
        },
        Op::Parameter(parameter) => {
            let new_param =
                Parameter::create(parameter.position, parameter.ty.clone(), parameter.span());
            let owner_id =
                owner_id_map.get(&parameter.owner_id).cloned().unwrap_or(parameter.owner_id);
            if let Some(mut param) = new_param.as_parameter_mut() {
                param.set_owner_id(owner_id);
                param.set_for_output(parameter.is_for_output);
            }
            new_param
        },
        Op::Value(value) => Value::create(value.value.clone()),
        Op::None(span) => Op::None(*span).into(),
    }
}

/// Helper used to duplicate nodes and their children recursively, used during Inlining and
/// Unrolling Additionally, if a Leaf is a `Parameter` that references the given owner id, it is
/// replaced with the corresponding item of the replace_parameter_list
/// Vec.
///
/// This is useful for inlining function calls (and replacing their parameters with the arguments of
/// the call) for Inlining, and for Unrolling loops (and replacing their parameters with the
/// iterator values) for Unrolling. Inlining: replace_parameter_list = arguments should be the
/// arguments from the `Call` Unrolling: replace_parameter_list =
/// self.for_inlining_context.unwrap().iterators
///
/// Note: The params_for_ref_node parameters is used to keep track of `Parameters`, and update their
/// owner ids as needed.
pub fn duplicate_node_or_replace(
    current_replace_map: &mut HashMap<usize, (Link<Op>, Link<Op>)>,
    node: Link<Op>,
    replace_parameter_list: Vec<Link<Op>>,
    ref_owner_id: OwnerId,
    params_for_ref_node: &mut HashMap<OwnerId, Vec<Link<Op>>>,
) {
    let mut interner = None;
    let mut owner_id_map: HashMap<OwnerId, OwnerId> = HashMap::new();
    let mut param_cache: HashMap<(OwnerId, usize, bool), Link<Op>> = HashMap::new();
    duplicate_node_or_replace_with_interner(
        current_replace_map,
        node,
        replace_parameter_list,
        ref_owner_id,
        params_for_ref_node,
        &mut owner_id_map,
        &mut param_cache,
        &mut interner,
    );
}

fn ensure_mapped(
    current_replace_map: &mut HashMap<usize, (Link<Op>, Link<Op>)>,
    child: &Link<Op>,
    replace_parameter_list: &Vec<Link<Op>>,
    ref_owner_id: OwnerId,
    params_for_ref_node: &mut HashMap<OwnerId, Vec<Link<Op>>>,
    owner_id_map: &mut HashMap<OwnerId, OwnerId>,
    param_cache: &mut HashMap<(OwnerId, usize, bool), Link<Op>>,
    interner: &mut Option<OpInterner>,
) -> Link<Op> {
    if let Some(entry) = current_replace_map.get(&child.get_ptr()) {
        return entry.1.clone();
    }
    duplicate_node_or_replace_with_interner(
        current_replace_map,
        child.clone(),
        replace_parameter_list.clone(),
        ref_owner_id,
        params_for_ref_node,
        owner_id_map,
        param_cache,
        interner,
    );
    current_replace_map
        .get(&child.get_ptr())
        .map(|entry| entry.1.clone())
        .unwrap_or_else(|| {
            panic!("missing replacement for child: {}", child.debug());
        })
}

/// Same as [duplicate_node_or_replace], but optionally interns high-fanout ops.
///
/// This is used in inlining/unrolling/index-projection paths to avoid allocating
/// duplicate subtrees when a shared op is safe to intern.
pub fn duplicate_node_or_replace_with_interner(
    current_replace_map: &mut HashMap<usize, (Link<Op>, Link<Op>)>,
    node: Link<Op>,
    replace_parameter_list: Vec<Link<Op>>,
    ref_owner_id: OwnerId,
    params_for_ref_node: &mut HashMap<OwnerId, Vec<Link<Op>>>,
    owner_id_map: &mut HashMap<OwnerId, OwnerId>,
    param_cache: &mut HashMap<(OwnerId, usize, bool), Link<Op>>,
    interner: &mut Option<OpInterner>,
) {
    match node.borrow().deref() {
        Op::Enf(enf) => {
            let expr = enf.expr.clone();
            let new_expr = ensure_mapped(
                current_replace_map,
                &expr,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_node = Enf::create(new_expr, enf.span(), enf.tag.clone());
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::Boundary(boundary) => {
            let expr = boundary.expr.clone();
            let kind = boundary.kind;
            let new_expr = ensure_mapped(
                current_replace_map,
                &expr,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_node = Boundary::create(new_expr, kind, boundary.span());
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::Add(add) => {
            let lhs = add.lhs.clone();
            let rhs = add.rhs.clone();
            let new_lhs_node = ensure_mapped(
                current_replace_map,
                &lhs,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_rhs_node = ensure_mapped(
                current_replace_map,
                &rhs,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_node = if let Some(interner) = interner.as_mut() {
                interner.intern_add(new_lhs_node, new_rhs_node, add.span())
            } else {
                Add::create(new_lhs_node, new_rhs_node, add.span())
            };
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::Sub(sub) => {
            let lhs = sub.lhs.clone();
            let rhs = sub.rhs.clone();
            let new_lhs_node = ensure_mapped(
                current_replace_map,
                &lhs,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_rhs_node = ensure_mapped(
                current_replace_map,
                &rhs,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_node = if let Some(interner) = interner.as_mut() {
                interner.intern_sub(new_lhs_node, new_rhs_node, sub.span())
            } else {
                Sub::create(new_lhs_node, new_rhs_node, sub.span())
            };
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::Mul(mul) => {
            let lhs = mul.lhs.clone();
            let rhs = mul.rhs.clone();
            let new_lhs_node = ensure_mapped(
                current_replace_map,
                &lhs,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_rhs_node = ensure_mapped(
                current_replace_map,
                &rhs,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_node = if let Some(interner) = interner.as_mut() {
                interner.intern_mul(new_lhs_node, new_rhs_node, mul.span())
            } else {
                Mul::create(new_lhs_node, new_rhs_node, mul.span())
            };
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::Exp(exp) => {
            let lhs = exp.lhs.clone();
            let rhs = exp.rhs.clone();
            let new_lhs_node = ensure_mapped(
                current_replace_map,
                &lhs,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_rhs_node = ensure_mapped(
                current_replace_map,
                &rhs,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_node = if let Some(interner) = interner.as_mut() {
                interner.intern_exp(new_lhs_node, new_rhs_node, exp.span())
            } else {
                Exp::create(new_lhs_node, new_rhs_node, exp.span())
            };
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::If(if_node) => {
            let match_arms = if_node.match_arms.clone();
            let old_owner_id = if_node.owner_id;
            let new_owner_id = OwnerId::next();
            owner_id_map.insert(old_owner_id, new_owner_id);
            let new_match_arms = match_arms
                .borrow()
                .iter()
                .cloned()
                .map(|arm| {
                    let new_expr = ensure_mapped(
                        current_replace_map,
                        &arm.expr,
                        &replace_parameter_list,
                        ref_owner_id,
                        params_for_ref_node,
                        owner_id_map,
                        param_cache,
                        interner,
                    );
                    let new_cond = ensure_mapped(
                        current_replace_map,
                        &arm.condition,
                        &replace_parameter_list,
                        ref_owner_id,
                        params_for_ref_node,
                        owner_id_map,
                        param_cache,
                        interner,
                    );
                    MatchArm::new(new_expr, new_cond)
                })
                .collect::<Vec<_>>();
            let new_node = If::create(new_match_arms, if_node.span());
            if let Some(mut if_mut) = new_node.as_if_mut() {
                if_mut.owner_id = new_owner_id;
            }
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::For(for_node) => {
            let iterators = for_node.iterators.clone();
            let body = for_node.expr.clone();
            let selector = for_node.selector.clone();
            let old_owner_id = for_node.owner_id;
            let new_owner_id = OwnerId::next();
            // Record owner-id remap early to keep parameter identity stable across the new For.
            owner_id_map.insert(old_owner_id, new_owner_id);
            // Use a local replace map and parameter cache so a duplicated For doesn't accidentally
            // reuse nodes from a different For context.
            let mut local_replace_map: HashMap<usize, (Link<Op>, Link<Op>)> = HashMap::new();
            let mut local_owner_id_map = owner_id_map.clone();
            let mut local_param_cache: HashMap<(OwnerId, usize, bool), Link<Op>> = HashMap::new();
            let new_iterators = iterators
                .borrow()
                .iter()
                .cloned()
                .map(|iterator| {
                    duplicate_node_or_replace_with_interner(
                        &mut local_replace_map,
                        iterator.clone(),
                        replace_parameter_list.clone(),
                        ref_owner_id,
                        params_for_ref_node,
                        &mut local_owner_id_map,
                        &mut local_param_cache,
                        interner,
                    );
                    local_replace_map
                        .get(&iterator.get_ptr())
                        .map(|(_, new_node)| new_node.clone())
                        .unwrap_or_else(|| iterator.clone())
                })
                .collect::<Vec<_>>()
                .into();
            duplicate_node_or_replace_with_interner(
                &mut local_replace_map,
                body.clone(),
                replace_parameter_list.clone(),
                ref_owner_id,
                params_for_ref_node,
                &mut local_owner_id_map,
                &mut local_param_cache,
                interner,
            );
            let new_body = local_replace_map
                .get(&body.get_ptr())
                .map(|(_, new_node)| new_node.clone())
                .unwrap_or_else(|| body.clone());
            let new_selector = if let Op::None(_) = selector.borrow().deref() {
                selector.clone()
            } else {
                duplicate_node_or_replace_with_interner(
                    &mut local_replace_map,
                    selector.clone(),
                    replace_parameter_list.clone(),
                    ref_owner_id,
                    params_for_ref_node,
                    &mut local_owner_id_map,
                    &mut local_param_cache,
                    interner,
                );
                local_replace_map
                    .get(&selector.get_ptr())
                    .map(|(_, new_node)| new_node.clone())
                    .unwrap_or_else(|| selector.clone())
            };
            let new_node = For::create(new_iterators, new_body, new_selector, for_node.span());
            if let Some(mut for_mut) = new_node.as_for_mut() {
                for_mut.owner_id = new_owner_id;
            }
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node.clone()));

            if let Some(params) = params_for_ref_node.remove(&old_owner_id) {
                // Any parameters that referenced the old For owner_id must be rebound to the
                // newly created For to avoid duplicate/aliased placeholders.
                // AIR_DEBUG_DUP_FOR traces param remapping during For duplication.
                let debug_dup_for = std::env::var("AIR_DEBUG_DUP_FOR").is_ok();
                if debug_dup_for {
                    eprintln!(
                        "duplicate_node_or_replace: updating {} params for for_owner_id={:?} -> new_owner_id={:?}",
                        params.len(),
                        old_owner_id,
                        new_owner_id
                    );
                    for param in params.iter() {
                        if let Some(param_ref) = param.as_parameter() {
                            eprintln!(
                                "duplicate_node_or_replace: param ptr={} pos={} is_for_output={} owner_id(before)={:?}",
                                param.get_ptr(),
                                param_ref.position,
                                param_ref.is_for_output,
                                param_ref.owner_id
                            );
                        }
                    }
                }
                for param in params.iter() {
                    param.as_parameter_mut().unwrap().set_owner_id(new_owner_id);
                    if let Some(param_ref) = param.as_parameter() {
                        let pos = param_ref.position;
                        let is_for_output = param_ref.is_for_output;
                        param_cache.remove(&(old_owner_id, pos, is_for_output));
                        param_cache.insert((new_owner_id, pos, is_for_output), param.clone());
                    }
                    if debug_dup_for {
                        if let Some(param_ref) = param.as_parameter() {
                            eprintln!(
                                "duplicate_node_or_replace: param ptr={} owner_id(after)={:?}",
                                param.get_ptr(),
                                param_ref.owner_id
                            );
                        }
                    }
                }

                params_for_ref_node.entry(new_owner_id).or_default().extend(params.into_iter());
            } else if std::env::var("AIR_DEBUG_DUP_FOR").is_ok() {
                eprintln!(
                    "duplicate_node_or_replace: no params found for for_owner_id={:?}",
                    old_owner_id
                );
            }
        },
        Op::Call(call) => {
            let arguments = call.arguments.clone();
            let function = call.function.clone();
            let new_arguments = arguments
                .borrow()
                .iter()
                .cloned()
                .map(|argument| {
                    ensure_mapped(
                        current_replace_map,
                        &argument,
                        &replace_parameter_list,
                        ref_owner_id,
                        params_for_ref_node,
                        owner_id_map,
                        param_cache,
                        interner,
                    )
                })
                .collect::<Vec<_>>();
            let new_node = Call::create(function, new_arguments, call.span());
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::Fold(fold) => {
            let iterator = fold.iterator.clone();
            let operator = fold.operator.clone();
            let initial_value = fold.initial_value.clone();
            let new_iterator = ensure_mapped(
                current_replace_map,
                &iterator,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_initial_value = ensure_mapped(
                current_replace_map,
                &initial_value,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_node = Fold::create(new_iterator, operator, new_initial_value, fold.span());
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::Vector(vector) => {
            let children_link = vector.children().clone();
            let children_ref = children_link.borrow();
            let children = children_ref.deref();
            let new_children = children
                .iter()
                .cloned()
                .map(|child| {
                    ensure_mapped(
                        current_replace_map,
                        &child,
                        &replace_parameter_list,
                        ref_owner_id,
                        params_for_ref_node,
                        owner_id_map,
                        param_cache,
                        interner,
                    )
                })
                .collect();
            let new_node = if let Some(interner) = interner.as_mut() {
                interner.intern_vector(new_children, vector.span())
            } else {
                Vector::create(new_children, vector.span())
            };
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::Matrix(matrix) => {
            let mut new_matrix = Vec::new();
            let children_link = matrix.children().clone();
            let children_ref = children_link.borrow();
            let children = children_ref.deref();
            for row in children.iter() {
                let row_children_link = row
                    .clone()
                    .as_vector()
                    .unwrap_or_else(|| panic!("expected Vector, found {row:?}"))
                    .children()
                    .clone();
                let row_children_ref = row_children_link.borrow();
                let row_children = row_children_ref.deref();
                let new_row_as_vec = row_children
                    .iter()
                    .cloned()
                    .map(|child| {
                        ensure_mapped(
                            current_replace_map,
                            &child,
                            &replace_parameter_list,
                            ref_owner_id,
                            params_for_ref_node,
                            owner_id_map,
                            param_cache,
                            interner,
                        )
                    })
                    .collect::<Vec<_>>();
                let new_row = Vector::create(new_row_as_vec, row.span());
                new_matrix.push(new_row);
            }
            let new_node = if let Some(interner) = interner.as_mut() {
                interner.intern_matrix(new_matrix, matrix.span())
            } else {
                Matrix::create(new_matrix, matrix.span())
            };
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::Accessor(accessor) => {
            let indexable = accessor.indexable.clone();
            let new_access_type = match accessor.access_type.clone() {
                MirAccessType::Default => MirAccessType::Default,
                MirAccessType::Index(index) => {
                    let new_index = ensure_mapped(
                        current_replace_map,
                        &index,
                        &replace_parameter_list,
                        ref_owner_id,
                        params_for_ref_node,
                        owner_id_map,
                        param_cache,
                        interner,
                    );
                    MirAccessType::Index(new_index)
                },
                MirAccessType::Matrix(row, col) => MirAccessType::Matrix(
                    ensure_mapped(
                        current_replace_map,
                        &row,
                        &replace_parameter_list,
                        ref_owner_id,
                        params_for_ref_node,
                        owner_id_map,
                        param_cache,
                        interner,
                    ),
                    ensure_mapped(
                        current_replace_map,
                        &col,
                        &replace_parameter_list,
                        ref_owner_id,
                        params_for_ref_node,
                        owner_id_map,
                        param_cache,
                        interner,
                    ),
                ),
            };
            let offset = accessor.offset;
            let new_indexable = ensure_mapped(
                current_replace_map,
                &indexable,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_node = if let Some(interner) = interner.as_mut() {
                interner.intern_accessor(new_indexable, new_access_type, offset, accessor.span())
            } else {
                Accessor::create(new_indexable, new_access_type, offset, accessor.span())
            };
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::BusOp(bus_op) => {
            let bus = bus_op.bus.clone();
            let kind = bus_op.kind;
            let args = bus_op.args.clone();
            let latch = bus_op.latch.clone();

            let new_args = args
                .iter()
                .cloned()
                .map(|arg| {
                    ensure_mapped(
                        current_replace_map,
                        &arg,
                        &replace_parameter_list,
                        ref_owner_id,
                        params_for_ref_node,
                        owner_id_map,
                        param_cache,
                        interner,
                    )
                })
                .collect();
            let new_latch = ensure_mapped(
                current_replace_map,
                &latch,
                &replace_parameter_list,
                ref_owner_id,
                params_for_ref_node,
                owner_id_map,
                param_cache,
                interner,
            );
            let new_node = BusOp::create(bus.clone(), kind, new_args, bus_op.span());

            // Update latch of cloned bus_op
            new_node.as_bus_op_mut().unwrap().latch = new_latch.clone();

            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::Parameter(parameter) => {
            let mut should_replace = parameter.owner_id == ref_owner_id;
            if should_replace && parameter.position >= replace_parameter_list.len() {
                // AIR_DEBUG_PARAM_REPLACE traces parameter replacement decisions.
                if std::env::var("AIR_DEBUG_PARAM_REPLACE").is_ok() {
                    eprintln!(
                        "duplicate_node_or_replace: param replace skipped pos={} owner_id={:?} ref_owner_id={:?} replace_len={}",
                        parameter.position,
                        parameter.owner_id,
                        ref_owner_id,
                        replace_parameter_list.len()
                    );
                }
                should_replace = false;
            }
            if !should_replace && std::env::var("AIR_DEBUG_PARAM_REPLACE").is_ok() {
                let mapped = owner_id_map.get(&parameter.owner_id).cloned();
                eprintln!(
                    "duplicate_node_or_replace: param not replaced pos={} owner_id={:?} ref_owner_id={:?} mapped_owner_id={:?}",
                    parameter.position, parameter.owner_id, ref_owner_id, mapped
                );
            }
            if should_replace && std::env::var("AIR_DEBUG_PARAM_REPLACE").is_ok() {
                eprintln!(
                    "duplicate_node_or_replace: param replaced ptr={} pos={} owner_id={:?} ref_owner_id={:?}",
                    node.get_ptr(),
                    parameter.position,
                    parameter.owner_id,
                    ref_owner_id
                );
            }
            if should_replace {
                let replace_by_node = replace_parameter_list[parameter.position].clone();
                if replace_by_node.as_parameter().is_some()
                    && std::env::var("AIR_DEBUG_PARAM_REPLACE").is_ok()
                {
                    let owner_id =
                        replace_by_node.as_parameter().map(|p| p.owner_id).unwrap_or_default();
                    eprintln!(
                        "duplicate_node_or_replace: replace_by_node is parameter owner_id={:?}",
                        owner_id
                    );
                }
                let new_node = if replace_by_node.as_parameter().is_some() {
                    // Preserve parameter identity.
                    replace_by_node
                } else if should_share_argument(&replace_by_node) {
                    replace_by_node
                } else {
                    // Duplicate argument once and reuse the mapped node for all parameter uses.
                    ensure_mapped(
                        current_replace_map,
                        &replace_by_node,
                        &replace_parameter_list,
                        ref_owner_id,
                        params_for_ref_node,
                        owner_id_map,
                        param_cache,
                        interner,
                    )
                };
                current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
            } else {
                let owner_id =
                    owner_id_map.get(&parameter.owner_id).cloned().unwrap_or(parameter.owner_id);
                // Parameter identity is keyed by (owner_id, position, is_for_output) so that
                // placeholders and "real" params never collide.
                let key = (owner_id, parameter.position, parameter.is_for_output);
                if let Some(existing) = param_cache.get(&key) {
                    // Reuse the cached parameter to preserve identity and avoid quadratic growth.
                    let entry = params_for_ref_node.entry(owner_id).or_default();
                    if !entry.iter().any(|p| p.get_ptr() == existing.get_ptr()) {
                        entry.push(existing.clone());
                    }
                    current_replace_map.insert(node.get_ptr(), (node.clone(), existing.clone()));
                    return;
                }
                let new_param =
                    Parameter::create(parameter.position, parameter.ty.clone(), parameter.span());
                if let Some(mut param) = new_param.as_parameter_mut() {
                    param.set_owner_id(owner_id);
                    param.set_for_output(parameter.is_for_output);
                }
                // AIR_DEBUG_PARAM_CREATE logs new parameter creation during duplication.
                if std::env::var("AIR_DEBUG_PARAM_CREATE").is_ok() {
                    eprintln!(
                        "duplicate_node_or_replace: created param ptr={} owner_id={:?}",
                        new_param.get_ptr(),
                        owner_id
                    );
                }
                param_cache.insert(key, new_param.clone());
                params_for_ref_node.entry(owner_id).or_default().push(new_param.clone());

                current_replace_map.insert(node.get_ptr(), (node.clone(), new_param));
            }
        },
        Op::Value(value) => {
            let new_node = if let Some(interner) = interner.as_mut() {
                interner.intern_value(value.value.clone())
            } else {
                Value::create(value.value.clone())
            };
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
        Op::None(span) => {
            let new_node = Op::None(*span).into();
            current_replace_map.insert(node.get_ptr(), (node.clone(), new_node));
        },
    }
}

/// Helper function to extract the constant felt value from a Link<Op> if it is one.
pub fn get_inner_const(value: &Link<Op>) -> Option<u64> {
    match value.borrow().deref() {
        Op::Value(v) => v.get_inner_const(),
        _ => None,
    }
}

/// Handle the visit of an accessor node, used for both Unrolling and ConstantPropagation passes
/// The `expect_constant_indices` bool indicates whether the indices need to be constant at
/// this stage.
pub fn handle_accessor_visit(
    accessor: Link<Op>,
    expect_constant_indices: bool,
    diagnostics: &DiagnosticsHandler,
) -> Result<Option<Link<Op>>, CompileError> {
    let Some(accessor_ref) = accessor.as_accessor() else {
        // This node may have been rewritten already; skip stale accessors.
        return Ok(None);
    };
    let indexable = accessor_ref.indexable.clone();
    let mir_access_type = accessor_ref.access_type.clone();
    let offset = accessor_ref.offset;

    match mir_access_type {
        // If we have a Default accessor, we add the row offset if needed, otherwise we just return
        // the indexable
        MirAccessType::Default => Ok(Some(add_row_offset_if_trace_access(&indexable, offset))),
        // If we have an Index accessor, we compute the index and query the index-th element of the
        // indexable. If the index is not a constant and computed_indices is true, we raise
        // a diagnostic. If the index is not a constant and computed_indices is false, we
        // keep the node as is. If the index is an out-of-bound constant, we raise a
        // diagnostic.
        MirAccessType::Index(index) => unroll_accessor_index_access_type(
            indexable,
            index,
            offset,
            expect_constant_indices,
            diagnostics,
        ),
        // If we have an Matrix accessor, we compute both the corresponding row and column, and
        // query the indexable accordingly. If either of row or col is not a constant and
        // computed_indices is true, we raise a diagnostic. If either of row or col is not a
        // constant and computed_indices is false, we keep the node as is. If either of row
        // or col is an out-of-bound constant, we raise a diagnostic.
        MirAccessType::Matrix(row, col) => unroll_accessor_matrix_access_type(
            indexable,
            row,
            col,
            expect_constant_indices,
            diagnostics,
        ),
    }
}

/// Large vectors/matrices can create huge graphs when fully unrolled; keep accessors instead.
pub const ACCESSOR_UNROLL_MAX_LEN: usize = 64;

/// Returns true if we should skip unrolling accessors over large vectors/matrices.
pub fn should_skip_accessor_unroll(indexable: &Link<Op>) -> bool {
    match indexable.borrow().deref() {
        Op::Vector(v) => {
            let len = v.children().borrow().len();
            len > ACCESSOR_UNROLL_MAX_LEN
        },
        Op::Matrix(m) => {
            let len = m.children().borrow().len();
            len > ACCESSOR_UNROLL_MAX_LEN
        },
        _ => false,
    }
}

/// Returns true if a node is large enough that it should be shared instead of duplicated.
pub fn should_share_argument(node: &Link<Op>) -> bool {
    match node.borrow().deref() {
        Op::Vector(v) => v.children().borrow().len() > ACCESSOR_UNROLL_MAX_LEN,
        Op::Matrix(m) => m.children().borrow().len() > ACCESSOR_UNROLL_MAX_LEN,
        Op::Accessor(accessor) => should_share_argument(&accessor.indexable),
        Op::Call(call) => {
            if let Some(func) = call.function.clone().as_function() {
                if let Some(ret_param) = func.return_type.as_parameter() {
                    return matches!(ret_param.ty, MirType::Vector(size) if size > ACCESSOR_UNROLL_MAX_LEN)
                        || matches!(ret_param.ty, MirType::Matrix(rows, _) if rows > ACCESSOR_UNROLL_MAX_LEN);
                }
            }
            false
        },
        _ => false,
    }
}

/// Helper function to unroll an Index accessor
fn unroll_accessor_index_access_type(
    indexable: Link<Op>,
    index: Link<Op>,
    accessor_offset: usize,
    expect_constant_indices: bool,
    diagnostics: &DiagnosticsHandler,
) -> Result<Option<Link<Op>>, CompileError> {
    let Some(index_usize) = extract_index_value(&index, expect_constant_indices, diagnostics)?
    else {
        return Ok(None);
    };
    if let Op::Vector(indexable_vector) = indexable.borrow().deref() {
        let indexable_children = indexable_vector.children();
        let indexable_vec = indexable_children.borrow();
        let child_accessed = match indexable_vec.get(index_usize) {
            Some(child_accessed) => child_accessed.clone(),
            None => {
                diagnostics
                    .diagnostic(miden_diagnostics::Severity::Error)
                    .with_message("attempted to access an index which is out of bounds")
                    .with_primary_label(index.span(), "index out of bounds")
                    .emit();
                return Err(CompileError::Failed);
            },
        };
        Ok(Some(add_row_offset_if_trace_access(&child_accessed, accessor_offset)))
    } else if let Some(value) = indexable.clone().as_value() {
        // If the indexable is either a PublicInput or a TraceAccess, we treat the index as
        // an offset
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
                Ok(Some(new_node))
            },
            MirValue::TraceAccess(trace_access) => {
                // We also need to account for the row offset
                let new_node = Value::create(SpannedMirValue {
                    span: value.value.span(),
                    value: MirValue::TraceAccess(TraceAccess {
                        segment: trace_access.segment,
                        column: trace_access.column + index_usize,
                        row_offset: trace_access.row_offset,
                    }),
                });
                Ok(Some(new_node))
            },
            _ => {
                unreachable!(
                    "Unexpected accessor, cannot have MirAccessType::Index with indexable {:?}",
                    indexable
                );
            },
        }
    } else {
        unreachable!(
            "Unexpected accessor, cannot have MirAccessType::Index with indexable {:?}",
            indexable
        );
    }
}

/// Helper function to unroll a Matrix accessor
fn unroll_accessor_matrix_access_type(
    indexable: Link<Op>,
    row: Link<Op>,
    col: Link<Op>,
    expect_constant_indices: bool,
    diagnostics: &DiagnosticsHandler,
) -> Result<Option<Link<Op>>, CompileError> {
    let Some(row_usize) = extract_index_value(&row, expect_constant_indices, diagnostics)? else {
        return Ok(None);
    };
    let Some(col_usize) = extract_index_value(&col, expect_constant_indices, diagnostics)? else {
        return Ok(None);
    };
    // Replace the current node by the index-th element of the vector
    // Raise diag if index is out of bounds
    if let Op::Vector(indexable_vector) = indexable.borrow().deref() {
        let indexable_children = indexable_vector.children();
        let indexable_vec = indexable_children.borrow();
        let row_accessed = match indexable_vec.get(row_usize) {
            Some(row_accessed) => row_accessed.clone(),
            None => {
                diagnostics
                    .diagnostic(miden_diagnostics::Severity::Error)
                    .with_message("attempted to access a row which is out of bounds")
                    .with_primary_label(row.span(), "row out of bounds")
                    .emit();
                return Err(CompileError::Failed);
            },
        };
        if let Op::Vector(row_accessed_vector) = row_accessed.borrow().deref() {
            let row_children = row_accessed_vector.children();
            let row_accessed_vec = row_children.borrow();
            let child_accessed = match row_accessed_vec.get(col_usize) {
                Some(child_accessed) => child_accessed.clone(),
                None => {
                    diagnostics
                        .diagnostic(miden_diagnostics::Severity::Error)
                        .with_message("attempted to access a col which is out of bounds")
                        .with_primary_label(col.span(), "col out of bounds")
                        .emit();
                    return Err(CompileError::Failed);
                },
            };
            Ok(Some(child_accessed))
        } else {
            unreachable!(
                "Unexpected accessor, cannot have MirAccessType::Matrix with indexable {:?}",
                indexable
            );
        }
    } else if let Op::Matrix(indexable_matrix) = indexable.borrow().deref() {
        let indexable_children = indexable_matrix.children();
        let indexable_vec = indexable_children.borrow();
        let row_accessed = match indexable_vec.get(row_usize) {
            Some(row_accessed) => row_accessed.clone(),
            None => {
                diagnostics
                    .diagnostic(miden_diagnostics::Severity::Error)
                    .with_message("attempted to access a row which is out of bounds")
                    .with_primary_label(row.span(), "row out of bounds")
                    .emit();
                return Err(CompileError::Failed);
            },
        };
        if let Op::Vector(row_accessed_vector) = row_accessed.borrow().deref() {
            let row_children = row_accessed_vector.children();
            let row_accessed_vec = row_children.borrow();
            let child_accessed = match row_accessed_vec.get(col_usize) {
                Some(child_accessed) => child_accessed.clone(),
                None => {
                    diagnostics
                        .diagnostic(miden_diagnostics::Severity::Error)
                        .with_message("attempted to access a col which is out of bounds")
                        .with_primary_label(col.span(), "col out of bounds")
                        .emit();
                    return Err(CompileError::Failed);
                },
            };
            Ok(Some(child_accessed))
        } else {
            unreachable!(
                "Unexpected accessor, cannot have MirAccessType::Matrix with indexable {:?}",
                indexable
            );
        }
    } else {
        unreachable!(
            "Unexpected accessor, cannot have MirAccessType::Matrix with indexable {:?}",
            indexable
        );
    }
}

/// Helper function to extract a usize value from an index expression with consistent error handling
///
/// Returns:
/// - `Ok(Some(usize))` - Successfully extracted constant value
/// - `Ok(None)` - Not a constant value but not required (expect_constant_indices=false)
/// - `Err(CompileError)` - Not a constant value when required (expect_constant_indices=true)
fn extract_index_value(
    index: &Link<Op>,
    expect_constant_indices: bool,
    diagnostics: &DiagnosticsHandler,
) -> Result<Option<usize>, CompileError> {
    match (get_inner_const(index), expect_constant_indices) {
        (Some(value), _) => Ok(Some(value as usize)),
        (None, true) => {
            diagnostics
                .diagnostic(miden_diagnostics::Severity::Error)
                .with_message("the index is not constant during constant propagation")
                .with_primary_label(index.span(), "index is not constant")
                .emit();
            Err(CompileError::Failed)
        },
        (None, false) => Ok(None),
    }
}

/// Helper function to add a row offset to a `TraceAccess` value, and return the node unchanged
/// otherwise.
fn add_row_offset_if_trace_access(node: &Link<Op>, offset: usize) -> Link<Op> {
    if let Some(value) = node.clone().as_value() {
        let mir_value = value.value.value.clone();
        if let MirValue::TraceAccess(trace_access) = mir_value {
            Value::create(SpannedMirValue {
                span: value.value.span(),
                value: MirValue::TraceAccess(TraceAccess {
                    segment: trace_access.segment,
                    column: trace_access.column,
                    row_offset: trace_access.row_offset + offset,
                }),
            })
        } else {
            node.clone()
        }
    } else {
        node.clone()
    }
}
