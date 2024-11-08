use std::collections::HashSet;

use crate::{Node, NodeIndex, Operation};

pub trait Graph {
    fn children(&self, node: &Operation) -> Vec<NodeIndex>;
    fn node(&self, node_index: &NodeIndex) -> &Node;
}
pub enum VisitOrder {
    DepthFirst,
    PostOrder,
}
pub trait VisitContext
where
    Self::Graph: Graph,
{
    type Graph;
    fn visit(&mut self, graph: &mut Self::Graph, node_index: NodeIndex);
    fn as_stack_mut(&mut self) -> &mut Vec<NodeIndex>;
    fn roots(&self, graph: &Self::Graph) -> HashSet<NodeIndex>;
    fn visit_order(&self) -> VisitOrder;
}

pub trait Visit: VisitContext {
    fn run(&mut self, graph: &mut Self::Graph);
    fn visit_postorder(&mut self, graph: &mut Self::Graph);
    fn visit_depthfirst(&mut self, graph: &mut Self::Graph);
    fn next_node(&mut self) -> Option<NodeIndex>;
    fn visit_later(&mut self, node_index: NodeIndex);
}
impl<T> Visit for T
where
    T: VisitContext,
    T::Graph: Graph,
{
    fn run(&mut self, graph: &mut Self::Graph) {
        match self.visit_order() {
            VisitOrder::DepthFirst => self.visit_depthfirst(graph),
            VisitOrder::PostOrder => self.visit_postorder(graph),
        }
        while let Some(node_index) = self.next_node() {
            self.visit(graph, node_index);
        }
    }
    fn visit_depthfirst(&mut self, graph: &mut Self::Graph) {
        for root_index in self.roots(graph).iter() {
            self.visit(graph, *root_index);
        }
    }
    fn visit_postorder(&mut self, graph: &mut Self::Graph) {
        for root_index in self.roots(graph).iter() {
            let mut stack = vec![*root_index];
            let mut node_index;
            let mut last: Option<NodeIndex> = None;
            while !stack.is_empty() {
                // safe to unwrap because stack is not empty
                node_index = *stack.last().unwrap();
                let node = graph.node(&node_index);
                let children = graph.children(&node.op);
                if children.is_empty() || last.is_some() && children.contains(&last.unwrap()) {
                    self.visit(graph, node_index);
                    stack.pop();
                    last = Some(node_index);
                } else {
                    for child in children.iter().rev() {
                        stack.push(*child);
                    }
                }
            }
        }
    }
    fn next_node(&mut self) -> Option<NodeIndex> {
        self.as_stack_mut().pop()
    }
    fn visit_later(&mut self, node_index: NodeIndex) {
        self.as_stack_mut().push(node_index);
    }
}
