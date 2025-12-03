extern crate alloc;
use alloc::collections::BTreeMap;

use mir::ir::QuadFelt;

use crate::graph::{AlgebraicGraph, Node, NodeIndex, Operation, RandomInputs};

impl AlgebraicGraph {
    /// Evaluates all the nodes in the graph at random points and returns a map of node indices to
    /// their evaluations.
    fn evaluate_on_random_inputs(&self) -> Vec<QuadFelt> {
        let mut random_inputs = RandomInputs::default();
        for index in 0..self.num_nodes() {
            let node_index = NodeIndex(index);
            random_inputs.eval(self, &node_index);
        }
        random_inputs.into_evaluations()
    }

    /// Given the evaluations of all nodes in the graph, ordered by their node indices, eliminates
    /// common subexpressions by replacing nodes with identical evaluations.
    ///
    /// In the process, as some nodes will be removed from the graph, node indices need to be
    /// remapped to keep consistent values (from `NodeIndex(0)` to `NodeIndex(self.num_nodes())`).
    /// This function returns the node indices remapping map.
    pub fn eliminate_common_subexpressions(&mut self) -> BTreeMap<NodeIndex, NodeIndex> {
        let evals = self.evaluate_on_random_inputs();

        let mut new_nodes = Vec::new();

        // 1. Keep track of evaluations, indices rewrites
        let mut evals_vec: Vec<QuadFelt> = Vec::with_capacity(evals.len());
        let mut renumbering_map: BTreeMap<NodeIndex, NodeIndex> = BTreeMap::new();

        for (index, eval) in evals.iter().enumerate() {
            let node_index = NodeIndex(index);

            // 2. For each node, check if its evaluation already exists in the map.
            if let Some(existing_index) = evals_vec.iter().position(|e| e == eval) {
                // 3. If it does, we will replace the node with the existing one
                renumbering_map.insert(node_index, NodeIndex(existing_index));
            } else {
                // 4. If it doesn't, rewrite the node indices if needed and add the node to the new
                //    graph
                evals_vec.push(*eval);

                let op = self.node(&node_index).op();
                let new_op = match op {
                    Operation::Value(_) => {
                        // Values do not need renumbering, they are leaf nodes
                        op.clone()
                    },
                    // Note: for Add, Sub, and Mul operations, we assume it's children have already
                    // been handled. This holds because when building the graph, we always insert
                    // children before parents, so their indices will always be lower and thus have
                    // been already processed and added to the `renumbering_map`.
                    Operation::Add(lhs, rhs) => {
                        let new_lhs = *renumbering_map.get(lhs).expect("Child of an operation not found in renumbering_map, but we should have already processed it");
                        let new_rhs = *renumbering_map.get(rhs).expect("Child of an operation not found in renumbering_map, but we should have already processed it");
                        Operation::Add(new_lhs, new_rhs)
                    },
                    Operation::Sub(lhs, rhs) => {
                        let new_lhs = *renumbering_map.get(lhs).expect("Child of an operation not found in renumbering_map, but we should have already processed it");
                        let new_rhs = *renumbering_map.get(rhs).expect("Child of an operation not found in renumbering_map, but we should have already processed it");
                        Operation::Sub(new_lhs, new_rhs)
                    },
                    Operation::Mul(lhs, rhs) => {
                        let new_lhs = *renumbering_map.get(lhs).expect("Child of an operation not found in renumbering_map, but we should have already processed it");
                        let new_rhs = *renumbering_map.get(rhs).expect("Child of an operation not found in renumbering_map, but we should have already processed it");
                        Operation::Mul(new_lhs, new_rhs)
                    },
                };
                let new_node = Node { op: new_op };
                let new_index = new_nodes.len();
                new_nodes.push(new_node);
                renumbering_map.insert(node_index, NodeIndex(new_index));
            }
        }

        // Replace the nodes in the graph with the new nodes
        self.nodes = new_nodes;

        renumbering_map
    }
}
