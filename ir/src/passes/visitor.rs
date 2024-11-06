use crate::{MirGraph, NodeIndex};

pub trait VisitContext {
    fn visit(&self, node_index: NodeIndex);
    fn as_stack_mut(&mut self) -> &mut Vec<NodeIndex>;
}

pub trait Visit: VisitContext {
    type Graph;
    fn run(&mut self, graph: &mut Self::Graph);
    fn next_node(&mut self) -> Option<NodeIndex>;
    fn visit_later(&mut self, node_index: NodeIndex);
}
impl<T> Visit for T
where
    T: VisitContext,
{
    type Graph = MirGraph;
    fn run(&mut self, graph: &mut Self::Graph) {
        for root_index in graph.roots.iter() {
            self.visit(root_index.clone());
        }
        while let Some(node_index) = self.next_node() {
            self.visit(node_index);
        }
    }
    fn next_node(&mut self) -> Option<NodeIndex> {
        self.as_stack_mut().pop()
    }
    fn visit_later(&mut self, node_index: NodeIndex) {
        self.as_stack_mut().push(node_index);
    }
}
