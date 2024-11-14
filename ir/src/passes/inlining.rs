use std::collections::{BTreeMap, HashSet};

use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{CompileError, Mir, MirGraph, NodeIndex, Operation};

use super::{visitor::VisitDefault, Visit, VisitContext, VisitOrder};

//pub struct Inlining<'a> {
//     #[allow(unused)]
//     diagnostics: &'a DiagnosticsHandler,
//}

pub struct Inlining {
    work_stack: Vec<NodeIndex>,
}

impl VisitContext for Inlining {
    type Graph = MirGraph;
    fn visit(&mut self, graph: &mut MirGraph, node_index: NodeIndex) {
        let node = graph.node(&node_index).clone();
        if let Operation::Definition(_, _, _) = node.op {
            self.visit_body(graph, node_index);
        }
    }
    fn as_stack_mut(&mut self) -> &mut Vec<NodeIndex> {
        &mut self.work_stack
    }
    fn boundary_roots(&self, graph: &MirGraph) -> HashSet<NodeIndex> {
        graph.boundary_constraints_roots.clone()
    }
    fn integrity_roots(&self, graph: &MirGraph) -> HashSet<NodeIndex> {
        graph.integrity_constraints_roots.clone()
    }
    fn visit_order(&self) -> VisitOrder {
        VisitOrder::Manual
    }
}

//impl<'p> Pass for Inlining<'p> {}
impl Pass for Inlining {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut context = Inlining::new();
        Visit::run(&mut context, &mut ir.constraint_graph_mut());
        Ok(ir)
    }
}

impl VisitDefault for Inlining {}

// impl<'a> Inlining<'a> {
//     pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
//         Self { diagnostics }
//         Self {}
//     }
// }
impl Inlining {
    pub fn new() -> Self {
        Self { work_stack: vec![] }
    }
    fn visit_body(&mut self, ir: &mut MirGraph, node_index: NodeIndex) {
        let def_node_index = node_index;
        let def_node = ir.node(&def_node_index).clone();
        if let Operation::Definition(_, _, body) = &def_node.op {
            // Find all calls in the body
            for (index_in_body, call_index) in body.iter().enumerate() {
                self.inline_call(ir, call_index, &def_node_index, index_in_body);
            }
        }
    }

    fn inline_call(
        &mut self,
        ir: &mut MirGraph,
        call_index: &NodeIndex,
        outer_def_index: &NodeIndex,
        index_in_body: usize,
    ) {
        let call_node = ir.node(call_index).clone();
        if let Operation::Call(def_index, arg_value_indexes) = &call_node.op {
            let mut body_index_map = BTreeMap::new();
            // Inline the body of the called function
            let new_nodes = self.inline_body(ir, &mut body_index_map, def_index, arg_value_indexes);
            let outer_def_node = ir.node(outer_def_index).clone();
            if let Operation::Definition(outer_func_arg_indexes, outer_func_ret, outer_body) =
                &outer_def_node.op
            {
                // Edit the body of the outer function
                // body.last: swap the call with the last node
                let mut new_body = outer_body.clone();
                new_body[index_in_body] = *new_nodes.last().unwrap();
                // body[..body.last]: insert the new nodes in reverse order
                for op_idx in new_nodes.iter().rev().skip(1) {
                    new_body.insert(index_in_body, *op_idx);
                }
                ir.update_node(
                    outer_def_index,
                    Operation::Definition(
                        outer_func_arg_indexes.clone(),
                        *outer_func_ret,
                        new_body,
                    ),
                );
                self.visit_later(*outer_def_index);
            }
        }
    }

    fn inline_body(
        &mut self,
        ir: &mut MirGraph,
        body_index_map: &mut BTreeMap<NodeIndex, NodeIndex>,
        def_index: &NodeIndex,
        arg_value_indexes: &[NodeIndex],
    ) -> Vec<NodeIndex> {
        let def_node = ir.node(def_index).clone();
        let mut new_body = vec![];
        if let Operation::Definition(arg_indexes, _, body) = &def_node.op {
            // map the arguments to the values of the call
            for (arg_index, arg_value_index) in arg_indexes.iter().zip(arg_value_indexes) {
                body_index_map.insert(*arg_index, *arg_value_index);
            }
            // Inline the body of the called function
            for node_index in body {
                self.inline_op(ir, body_index_map, node_index, &mut new_body);
            }
        }
        new_body
    }

    fn inline_op(
        &mut self,
        ir: &mut MirGraph,
        body_index_map: &mut BTreeMap<NodeIndex, NodeIndex>,
        op_index: &NodeIndex,
        new_body: &mut Vec<NodeIndex>,
    ) {
        // Clone the operation and insert it in the new body
        let new_node = ir.insert_op_placeholder();
        body_index_map.insert(*op_index, new_node);
        let op_node = ir.node(op_index).clone();
        // Update the operation with the new indexes
        let op = match op_node.op.clone() {
            Operation::Value(value) => Operation::Value(value),
            Operation::Add(lhs, rhs) => Operation::Add(
                *body_index_map.get(&lhs).expect("Add lhs not found"),
                *body_index_map.get(&rhs).expect("Add rhs not found"),
            ),
            Operation::Sub(lhs, rhs) => Operation::Sub(
                *body_index_map.get(&lhs).expect("Sub lhs not found"),
                *body_index_map.get(&rhs).expect("Sub rhs not found"),
            ),
            Operation::Mul(lhs, rhs) => Operation::Mul(
                *body_index_map.get(&lhs).expect("Mul lhs not found"),
                *body_index_map.get(&rhs).expect("Mul rhs not found"),
            ),
            Operation::Vector(values) => Operation::Vector(
                values
                    .iter()
                    .map(|value_index| {
                        *body_index_map
                            .get(value_index)
                            .expect("Vector value not found")
                    })
                    .collect(),
            ),
            Operation::Matrix(rows) => Operation::Matrix(
                rows.iter()
                    .map(|row| {
                        row.iter()
                            .map(|value_index| {
                                *body_index_map
                                    .get(value_index)
                                    .expect("Matrix value not found")
                            })
                            .collect()
                    })
                    .collect(),
            ),
            Operation::Call(def_index, arg_value_indexes) => Operation::Call(
                def_index,
                arg_value_indexes
                    .iter()
                    .map(|arg_value_index| {
                        *body_index_map
                            .get(arg_value_index)
                            .unwrap_or(arg_value_index)
                    })
                    .collect(),
            ),
            Operation::If(cond, then_index, else_index) => Operation::If(
                *body_index_map.get(&cond).unwrap_or(&cond),
                *body_index_map.get(&then_index).unwrap_or(&then_index),
                *body_index_map.get(&else_index).unwrap_or(&else_index),
            ),
            Operation::For(iterators, body_index, opt_selector) => Operation::For(
                iterators
                    .iter()
                    .map(|iterator_index| {
                        *body_index_map.get(iterator_index).unwrap_or(iterator_index)
                    })
                    .collect(),
                *body_index_map.get(&body_index).unwrap_or(&body_index),
                opt_selector.map(|selector_index| {
                    *body_index_map
                        .get(&selector_index)
                        .unwrap_or(&selector_index)
                }),
            ),
            Operation::Fold(iterator_index, fold_op, init_index) => Operation::Fold(
                *body_index_map
                    .get(&iterator_index)
                    .unwrap_or(&iterator_index),
                fold_op,
                *body_index_map.get(&init_index).unwrap_or(&init_index),
            ),
            Operation::Enf(value_index) => {
                Operation::Enf(*body_index_map.get(&value_index).unwrap_or(&value_index))
            }
            Operation::Boundary(boundary, value_index) => Operation::Boundary(
                boundary,
                *body_index_map.get(&value_index).unwrap_or(&value_index),
            ),
            Operation::Variable(var) => Operation::Variable(var),
            Operation::Definition(_, _, _) => unreachable!(),
            Operation::Placeholder => Operation::Placeholder,
        };
        ir.update_node(&new_node, op);
        new_body.push(new_node);
    }
}
