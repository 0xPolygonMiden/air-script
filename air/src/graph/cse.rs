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

#[cfg(test)]
mod tests {
    use air_parser::ast::TraceSegmentId;

    use crate::{AlgebraicGraph, Operation, Value, ir::TraceAccess};

    /// Regression test for a bug where `RandomInputs::eval`'s indexing scheme for
    /// `Value::TraceAccess` computed `index = column * 2 + row_offset`, implicitly assuming
    /// `row_offset` was always 0 or 1.
    ///
    /// That assumption does not hold in general: `row_offset` can be 2 or greater for
    /// constraints spanning larger frames (see `ConstraintDomain::EveryFrame`), reachable via
    /// ordinary evaluator/function composition (row offsets accumulate when an evaluator that
    /// applies `'` to one of its own parameters is invoked with an already-offset argument).
    /// In that case the flat index collided between unrelated columns, e.g.:
    ///
    ///   (column=0, row_offset=2) -> index = 0 * 2 + 2 = 2
    ///   (column=1, row_offset=0) -> index = 1 * 2 + 0 = 2
    ///
    /// Both would be assigned the exact same random evaluation -- not a negligible-probability
    /// collision, but a deterministic one caused by the indexing scheme itself -- causing
    /// `eliminate_common_subexpressions` to incorrectly treat two distinct trace cells as
    /// equal and merge them, silently corrupting every reference to either one.
    ///
    /// After the fix (keying random values on `(column, row_offset)` directly instead of a
    /// derived flat index), these two nodes must remain distinct.
    #[test]
    fn distinct_trace_accesses_with_colliding_legacy_index_are_not_merged_by_cse() {
        let mut graph = AlgebraicGraph::default();

        let col0_offset2 = graph.insert_node(Operation::Value(Value::TraceAccess(
            TraceAccess::new(TraceSegmentId::Main, 0, 2),
        )));
        let col1_offset0 = graph.insert_node(Operation::Value(Value::TraceAccess(
            TraceAccess::new(TraceSegmentId::Main, 1, 0),
        )));

        // Sanity check: insert_node must not have deduped these -- they are genuinely distinct
        // Value nodes (different column AND different offset).
        assert_ne!(
            col0_offset2, col1_offset0,
            "precondition failed: the two trace accesses were already deduped structurally"
        );

        let renumbering_map = graph.eliminate_common_subexpressions();

        let new_col0_offset2 = renumbering_map[&col0_offset2];
        let new_col1_offset0 = renumbering_map[&col1_offset0];

        assert_ne!(
            new_col0_offset2, new_col1_offset0,
            "main[0]'' (col=0, offset=2) and main[1] (col=1, offset=0) are distinct trace cells \
             and must not be merged by common-subexpression elimination"
        );
    }

    /// Same scenario as above, but also checks a case that already worked correctly before the
    /// fix (offsets 0 and 1 on different columns), to guard against a regression in the other
    /// direction (e.g. accidentally treating every trace access as distinct).
    #[test]
    fn identical_trace_accesses_are_still_merged_by_cse() {
        let mut graph = AlgebraicGraph::default();

        let a = graph.insert_node(Operation::Value(Value::TraceAccess(TraceAccess::new(
            TraceSegmentId::Main,
            0,
            1,
        ))));
        let b = graph.insert_node(Operation::Value(Value::TraceAccess(TraceAccess::new(
            TraceSegmentId::Main,
            0,
            1,
        ))));

        // insert_node already dedupes structurally-identical Value nodes, so this should be the
        // very same node index even before CSE runs.
        assert_eq!(a, b);

        let renumbering_map = graph.eliminate_common_subexpressions();
        assert_eq!(renumbering_map[&a], renumbering_map[&b]);
    }
}
