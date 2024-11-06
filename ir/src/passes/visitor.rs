use std::collections::HashSet;

use crate::NodeIndex;

pub trait VisitContext {
    type Graph;
    fn visit(&mut self, graph: &mut Self::Graph, node_index: NodeIndex);
    fn as_stack_mut(&mut self) -> &mut Vec<NodeIndex>;
    fn roots(&self, graph: &Self::Graph) -> HashSet<NodeIndex>;
}

pub trait Visit: VisitContext {
    fn run(&mut self, graph: &mut Self::Graph);
    fn next_node(&mut self) -> Option<NodeIndex>;
    fn visit_later(&mut self, node_index: NodeIndex);
}
impl<T> Visit for T
where
    T: VisitContext,
{
    fn run(&mut self, graph: &mut Self::Graph) {
        for root_index in self.roots(graph).iter() {
            self.visit(graph, *root_index);
        }
        while let Some(node_index) = self.next_node() {
            self.visit(graph, node_index);
        }
    }
    fn next_node(&mut self) -> Option<NodeIndex> {
        self.as_stack_mut().pop()
    }
    fn visit_later(&mut self, node_index: NodeIndex) {
        self.as_stack_mut().push(node_index);
    }
}
