extern crate petgraph;
use petgraph::graph::Graph;
use petgraph::dot::Dot;
use petgraph::stable_graph::NodeIndex;

use crate::AirIR;

pub fn generate_graph_vis(ir: &AirIR) -> String {
    for trace_segment in 0..ir.segment_widths().len() {
        let boundary_constraints = ir.boundary_constraints(trace_segment)
        let mut g: Graph<&str, &str> = Graph::new();
        let mut nodes: Vec<NodeIndex> = Vec::new();
        nodes.push(g.add_node("a"));
        nodes.push(g.add_node("b"));
        g.add_edge(nodes[0], nodes[1], "a to b");
        Dot::new(&g).to_string()
    }
}