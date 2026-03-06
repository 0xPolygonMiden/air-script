//! Unrolling second pass.
//!
//! Inlines `For` bodies using the contexts captured in the first pass while
//! preserving parameter identity.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    ops::Deref,
};

use miden_diagnostics::{DiagnosticsHandler, Spanned};

use crate::{
    CompileError,
    ir::{Graph, Link, Mul, Node, Op, OpInterner, OwnerId, Parent, Vector},
    passes::{
        Visitor, duplicate_node, duplicate_node_or_replace_with_interner,
        unrolling::ForInliningContext,
    },
};

/// Second pass of unrolling: inline `For` bodies and fix up parameters.
pub struct UnrollingSecondPass<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,
    // A list of all the children of `For` nodes to inline
    bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,
    // The current context for inlining a `For` node, if any
    for_inlining_context: Option<ForInliningContext>,
    // A map of nodes to replace with their inlined version
    nodes_to_replace: HashMap<usize, (Link<Op>, Link<Op>)>,
    // We keep track of all parameters referencing a given `For` node
    params_for_ref_node: HashMap<OwnerId, Vec<Link<Op>>>,
    // Owner remaps when duplicating nested For/If nodes
    owner_id_map: HashMap<OwnerId, OwnerId>,
    // Cache parameters by (owner_id, position, is_for_output) to preserve identity across
    // duplication and avoid re-allocating equivalent params in nested contexts.
    param_cache: HashMap<(OwnerId, usize, bool), Link<Op>>,
    // Template contexts keyed by (owner_id, position) for nested duplication.
    context_by_owner_pos: HashMap<(OwnerId, usize), ForInliningContext>,
}

impl<'a> UnrollingSecondPass<'a> {
    /// Construct a new second-pass unroller.
    pub fn new(
        diagnostics: &'a DiagnosticsHandler,
        bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,
    ) -> Self {
        Self {
            diagnostics,
            work_stack: vec![],
            bodies_to_inline,
            for_inlining_context: None,
            nodes_to_replace: HashMap::new(),
            params_for_ref_node: HashMap::new(),
            owner_id_map: HashMap::new(),
            param_cache: HashMap::new(),
            context_by_owner_pos: HashMap::new(),
        }
    }
}

impl Visitor for UnrollingSecondPass<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }

    // The root nodes visited during the second pass are the children of `For` nodes to inline
    fn root_nodes_to_visit(&self, _graph: &Graph) -> Vec<Link<Node>> {
        self.bodies_to_inline
            .iter()
            .map(|(k, _v)| k)
            .cloned()
            .map(|op| op.as_node())
            .collect::<Vec<_>>()
    }

    fn run(&mut self, graph: &mut Graph) -> Result<(), CompileError> {
        let mut seen_params: HashSet<usize> = HashSet::new();
        self.context_by_owner_pos.clear();
        // Seed a template context per (owner_id, position) so nested unrolling can
        // reconstruct the right body/iterators after duplication.
        for (param, ctx) in self.bodies_to_inline.iter() {
            if let Some(param_ref) = param.as_parameter() {
                self.context_by_owner_pos
                    .entry((param_ref.owner_id, param_ref.position))
                    .or_insert_with(|| ctx.clone());
            }
        }

        let reachable_params = self.collect_reachable_params_by_key(graph);
        let mut queue: VecDeque<(Link<Op>, ForInliningContext)> = VecDeque::new();
        for (param, ctx) in self.bodies_to_inline.iter() {
            if let Some(param_ref) = param.as_parameter() {
                let key = (param_ref.owner_id, param_ref.position);
                if let Some(params) = reachable_params.get(&key) {
                    // Prefer reachable parameters in the live graph; this avoids queuing
                    // stale placeholders and keeps unrolling bounded.
                    for actual_param in params.iter() {
                        let ptr = actual_param.get_ptr();
                        if !seen_params.insert(ptr) {
                            continue;
                        }
                        queue.push_back((actual_param.clone(), ctx.clone()));
                    }
                    if !params.is_empty() {
                        continue;
                    }
                }
            }
            // Fall back to the placeholder param if no reachable instance was found.
            let ptr = param.get_ptr();
            if !seen_params.insert(ptr) {
                continue;
            }
            queue.push_back((param.clone(), ctx.clone()));
        }

        while let Some((root_param, ctx)) = queue.pop_front() {
            // Skip stale or already-replaced placeholders.
            if root_param.as_parameter().is_none() {
                continue;
            }

            // Set the context corresponding to the `For` node we are inlining.
            self.for_inlining_context = Some(ctx.clone());
            self.nodes_to_replace.clear();
            self.params_for_ref_node.clear();
            self.owner_id_map.clear();
            self.param_cache.clear();
            // Recursively scan the body of the `For` node to inline
            self.scan_node(graph, self.for_inlining_context.clone().unwrap().body.as_node())?;
            while let Some(node) = self.work_stack().pop() {
                self.visit_node(graph, node.clone())?;
            }

            // We have finished inlining the body, we can now replace the Root node with the body
            let body = self.for_inlining_context.clone().unwrap().body;
            if !self.nodes_to_replace.contains_key(&body.get_ptr()) {
                let mut interner: Option<OpInterner> = None;
                duplicate_node_or_replace_with_interner(
                    &mut self.nodes_to_replace,
                    body.clone(),
                    self.for_inlining_context.clone().unwrap().iterators.clone(),
                    self.for_inlining_context.clone().unwrap().ref_owner_id,
                    &mut self.params_for_ref_node,
                    &mut self.owner_id_map,
                    &mut self.param_cache,
                    &mut interner,
                );
            }
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
                                root_param.span(),
                            );
                            new_vec.push(new_node_child_with_selector);
                        }
                        Vector::create(new_vec, root_param.span())
                    } else {
                        Mul::create(selector, new_node, root_param.span())
                    }
                } else {
                    new_node
                };

            // Update the root node with the new inlined body and reset the context to None
            root_param.set(&new_node_with_selector_if_needed);
            self.for_inlining_context = None;

            // Enqueue contexts for any duplicated placeholder params (nested comprehensions).
            self.enqueue_nested_contexts(&ctx, &mut queue, &mut seen_params);
        }

        Ok(())
    }

    fn visit_node(&mut self, _graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        // Skip stale nodes
        if node.is_stale() {
            return Ok(());
        }

        // visit_node is called on all the nodes in the body of a `For` node, they should never be
        // Root nodes
        let op = node.clone().as_op().expect("UnrollingSecondPass::visit_node on a non-Op node");

        // Will duplicate the body of the `For` node, replacing the corresponding `For` node's
        // Parameters by the values taken by iterators. Other Parameters will not be
        // replaced (in case of nested `For` nodes)
        let mut interner: Option<OpInterner> = None;
        duplicate_node_or_replace_with_interner(
            &mut self.nodes_to_replace,
            op,
            self.for_inlining_context.clone().unwrap().iterators.clone(),
            self.for_inlining_context.clone().unwrap().ref_owner_id,
            &mut self.params_for_ref_node,
            &mut self.owner_id_map,
            &mut self.param_cache,
            &mut interner,
        );
        Ok(())
    }
}

impl<'a> UnrollingSecondPass<'a> {
    /// Enqueue nested `For` contexts discovered during duplication.
    ///
    /// This preserves parameter identity for nested comprehensions by mapping placeholder
    /// parameters back to their template contexts.
    fn enqueue_nested_contexts(
        &mut self,
        outer_ctx: &ForInliningContext,
        queue: &mut VecDeque<(Link<Op>, ForInliningContext)>,
        seen_params: &mut HashSet<usize>,
    ) {
        let mut to_enqueue: Vec<(Link<Op>, ForInliningContext)> = Vec::new();
        // Build reverse owner_id mapping so we can recover the template context even when a
        // nested duplication creates a fresh owner id.
        let mut reverse_owner_id_map: HashMap<OwnerId, OwnerId> = HashMap::new();
        for (old_owner, new_owner) in self.owner_id_map.iter() {
            reverse_owner_id_map.insert(*new_owner, *old_owner);
        }

        for (_orig_ptr, (_orig, new_node)) in self.nodes_to_replace.iter() {
            let Some(param_ref) = new_node.as_parameter() else {
                continue;
            };
            // Only For-output placeholders carry nested unrolling contexts.
            if !param_ref.is_for_output {
                continue;
            }
            let key = (param_ref.owner_id, param_ref.position);
            let template_ctx = if let Some(ctx) = self.context_by_owner_pos.get(&key).cloned() {
                Some(ctx)
            } else if let Some(old_owner_id) = reverse_owner_id_map.get(&param_ref.owner_id) {
                self.context_by_owner_pos.get(&(*old_owner_id, param_ref.position)).cloned()
            } else {
                None
            };
            let Some(template_ctx) = template_ctx else {
                continue;
            };
            let new_ptr = new_node.get_ptr();
            if seen_params.contains(&new_ptr) {
                continue;
            }
            // Clone the inner context while substituting the outer iterators so nested For bodies
            // are re-bound to the correct iteration values.
            let new_ctx =
                self.duplicate_context_with_outer(&template_ctx, outer_ctx, param_ref.owner_id);
            seen_params.insert(new_ptr);
            to_enqueue.push((new_node.clone(), new_ctx));
        }

        for (param, ctx) in to_enqueue.into_iter() {
            queue.push_back((param, ctx));
        }
    }

    /// Clone an inner `For` context while substituting outer iterators.
    ///
    /// This re-binds the inner body/iterators to the outer loop values and remaps owner ids.
    fn duplicate_context_with_outer(
        &self,
        inner_ctx: &ForInliningContext,
        outer_ctx: &ForInliningContext,
        new_ref_owner_id: OwnerId,
    ) -> ForInliningContext {
        let mut nodes_to_replace: HashMap<usize, (Link<Op>, Link<Op>)> = HashMap::new();
        let mut params_for_ref_node: HashMap<OwnerId, Vec<Link<Op>>> = HashMap::new();
        let mut owner_id_map: HashMap<OwnerId, OwnerId> = HashMap::new();
        let mut param_cache: HashMap<(OwnerId, usize, bool), Link<Op>> = HashMap::new();
        let mut interner: Option<OpInterner> = None;

        // Remap the inner For owner_id to the duplicated owner so parameter identity stays stable.
        owner_id_map.insert(inner_ctx.ref_owner_id, new_ref_owner_id);

        // Replace inner parameters with outer iterators (captures the outer loop variables).
        let replace_list = outer_ctx.iterators.clone();
        let ref_owner_id = outer_ctx.ref_owner_id;

        duplicate_node_or_replace_with_interner(
            &mut nodes_to_replace,
            inner_ctx.body.clone(),
            replace_list.clone(),
            ref_owner_id,
            &mut params_for_ref_node,
            &mut owner_id_map,
            &mut param_cache,
            &mut interner,
        );
        let new_body = nodes_to_replace
            .get(&inner_ctx.body.get_ptr())
            .map(|(_, new_node)| new_node.clone())
            .unwrap_or_else(|| inner_ctx.body.clone());

        let mut new_iterators = Vec::with_capacity(inner_ctx.iterators.len());
        for iterator in inner_ctx.iterators.iter() {
            if nodes_to_replace.contains_key(&iterator.get_ptr()) {
                new_iterators.push(nodes_to_replace.get(&iterator.get_ptr()).unwrap().1.clone());
            } else {
                duplicate_node_or_replace_with_interner(
                    &mut nodes_to_replace,
                    iterator.clone(),
                    replace_list.clone(),
                    ref_owner_id,
                    &mut params_for_ref_node,
                    &mut owner_id_map,
                    &mut param_cache,
                    &mut interner,
                );
                new_iterators.push(
                    nodes_to_replace
                        .get(&iterator.get_ptr())
                        .map(|(_, new_node)| new_node.clone())
                        .unwrap_or_else(|| iterator.clone()),
                );
            }
        }

        let new_selector = inner_ctx.selector.as_ref().map(|selector| {
            if nodes_to_replace.contains_key(&selector.get_ptr()) {
                nodes_to_replace.get(&selector.get_ptr()).unwrap().1.clone()
            } else {
                duplicate_node_or_replace_with_interner(
                    &mut nodes_to_replace,
                    selector.clone(),
                    replace_list.clone(),
                    ref_owner_id,
                    &mut params_for_ref_node,
                    &mut owner_id_map,
                    &mut param_cache,
                    &mut interner,
                );
                nodes_to_replace
                    .get(&selector.get_ptr())
                    .map(|(_, new_node)| new_node.clone())
                    .unwrap_or_else(|| selector.clone())
            }
        });

        ForInliningContext {
            body: new_body,
            iterators: new_iterators,
            selector: new_selector,
            ref_owner_id: new_ref_owner_id,
        }
    }

    /// Collect reachable `For` output placeholder parameters keyed by (owner_id, position).
    fn collect_reachable_params_by_key(
        &self,
        graph: &Graph,
    ) -> HashMap<(OwnerId, usize), Vec<Link<Op>>> {
        let roots = crate::ir::extract_all_roots(graph);
        let mut seen: HashSet<usize> = HashSet::new();
        let mut stack: Vec<Link<Node>> = roots;
        let mut found: HashMap<(OwnerId, usize), Vec<Link<Op>>> = HashMap::new();

        while let Some(node) = stack.pop() {
            let ptr = node.get_ptr();
            if !seen.insert(ptr) {
                continue;
            }
            if let Node::Parameter(param_back) = &*node.borrow()
                && let Some(param_op) = param_back.to_link()
                && let Some(param_ref) = param_op.as_parameter()
            {
                if !param_ref.is_for_output {
                    continue;
                }
                let key = (param_ref.owner_id, param_ref.position);
                if self.context_by_owner_pos.contains_key(&key) {
                    found.entry(key).or_default().push(param_op.clone());
                }
            }
            if node.as_owner().is_some() {
                for child in node.children().borrow().iter() {
                    stack.push(child.clone().as_node());
                }
            }
        }

        found
    }
}
