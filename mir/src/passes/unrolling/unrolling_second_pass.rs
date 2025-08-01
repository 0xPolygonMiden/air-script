use std::{collections::HashMap, ops::Deref, rc::Rc};

use miden_diagnostics::{DiagnosticsHandler, Spanned};

use crate::{
    CompileError,
    ir::{Graph, Link, Mul, Node, Op, Owner, Parent, Vector},
    passes::{Visitor, duplicate_node, duplicate_node_or_replace, unrolling::ForInliningContext},
};

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
            // Set the context corresponding to the For node we are inlining
            self.set_context(root);

            // Recursively scan the body of the For node to inline
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

            // Update the root node with the new inlined body and reset the context to None
            root.as_op().unwrap().set(&new_node_with_selector_if_needed);
            self.for_inlining_context = None;
        }

        Ok(())
    }
    fn visit_node(&mut self, _graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        // Skip stale nodes
        if node.is_stale() {
            return Ok(());
        }

        // visit_node is called on all the nodes in the body of a For node, they should never be
        // Root nodes
        let Some(op) = node.clone().as_op() else {
            unreachable!("UnrollingSecondPass::visit_node on a non-Op node: {:?}", node);
        };

        // Will duplicate the body of the For node, replacing the corresponding For node's
        // Parameters by the values taken by iterators. Other Parameters will not be
        // replaced (in case of nested For nodes)
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
        Ok(())
    }
}

impl<'a> UnrollingSecondPass<'a> {
    /// Sets the context for inlining a For node based on the root node.
    fn set_context(&mut self, root: &Link<Node>) {
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
    }
}
