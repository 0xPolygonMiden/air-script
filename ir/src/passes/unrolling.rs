use std::{collections::{BTreeMap, HashMap, HashSet}, f32::consts::E, mem, ops::ControlFlow};

use air_parser::ast::Boundary;
use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{CompileError, ConstantValue, FoldOperator, Mir, MirGraph, MirType, MirValue, NodeIndex, Operation, SpannedMirValue, SpannedVariable, TraceAccess};

use super::{Visit, VisitContext, VisitOrder};

//pub struct Unrolling<'a> {
//     #[allow(unused)]
//     diagnostics: &'a DiagnosticsHandler,
//}

#[derive(Clone, Default)]
pub struct ForInliningContext {
    body_index: NodeIndex,
    iterators: Vec<NodeIndex>,
    selector: Option<NodeIndex>,
    index: usize,
    parent_for: NodeIndex,
}

impl ForInliningContext {}

pub struct Unrolling {
    // general context
    work_stack: Vec<NodeIndex>,
    during_first_pass: bool,

    // context for both passes
    bodies_to_inline: HashMap<NodeIndex, ForInliningContext>,

    // context for second pass
    for_inlining_context: ForInliningContext,
    nodes_to_replace: HashMap<NodeIndex, NodeIndex>,
}

//impl<'p> Pass for Unrolling<'p> {}
impl Pass for Unrolling {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        match self.run_visitor(&mut ir.constraint_graph_mut()) {
            ControlFlow::Continue(()) => Ok(ir),
            ControlFlow::Break(_err) => Err(CompileError::Failed),
        }
    }
}

impl Visit for Unrolling {

    fn run(&mut self, graph: &mut Self::Graph) {

        // First pass, unroll all nodes fully, except for For nodes
        self.during_first_pass = true;
        match self.visit_order() {
            VisitOrder::Manual => self.visit_manual(graph),
            VisitOrder::PostOrder => self.visit_postorder(graph),
            VisitOrder::DepthFirst => self.visit_depthfirst(graph),
        }
        while let Some(node_index) = self.next_node() {
            self.visit(graph, node_index);
        }

        // Second pass, inline For nodes
        self.during_first_pass = false;
        match self.visit_order() {
            VisitOrder::Manual => self.visit_manual(graph),
            VisitOrder::PostOrder => self.visit_postorder(graph),
            VisitOrder::DepthFirst => self.visit_depthfirst(graph),
        }
        while let Some(node_index) = self.next_node() {
            self.visit(graph, node_index);
        }
    }

}

// impl<'a> Unrolling<'a> {
//     pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
//         Self { diagnostics }
//         Self {}
//     }
// }
impl Unrolling {
    pub fn new() -> Self {
        Self { 
            work_stack: vec![],
            during_first_pass: true,
            bodies_to_inline: HashMap::new(),
            for_inlining_context: ForInliningContext::default(),
            nodes_to_replace: HashMap::new(),
        }
    }
    //TODO MIR: Implement inlining pass on MIR
    // 1. Understand the basics of the previous inlining process
    // 2. Remove what is done during lowering from AST to MIR (unroll, ...)
    // 3. Check how it translates to the MIR structure
    fn run_visitor(&mut self, ir: &mut MirGraph) -> ControlFlow<()> {
        Visit::run(self, ir);
        ControlFlow::Continue(())
    }
}

enum BinaryOp {
    Add,
    Sub,
    Mul,
}

impl Unrolling {
    fn visit_value(&mut self, graph: &mut MirGraph, node_index: NodeIndex, spanned_mir_value: SpannedMirValue) {

        match spanned_mir_value.value {
            MirValue::Constant(c) => match c {
                ConstantValue::Felt(_) => { },
                ConstantValue::Vector(v) => {
                    let mut vec = vec![];
                    for val in v {
                        let val = graph.insert_op_value(SpannedMirValue {
                            span: spanned_mir_value.span.clone(),
                            value: MirValue::Constant(ConstantValue::Felt(val)),
                        });
                        vec.push(val);
                    }
                    graph.update_node(&node_index, Operation::Vector(vec));
                },
                ConstantValue::Matrix(m) => {
                    let mut res_m = vec![];
                    for row in m {
                        let mut res_row = vec![];
                        for val in row {
                            let val = graph.insert_op_value(SpannedMirValue {
                                span: spanned_mir_value.span.clone(),
                                value: MirValue::Constant(ConstantValue::Felt(val)),
                            });
                            res_row.push(val);
                        }
                        res_m.push(res_row);
                    }
                    graph.update_node(&node_index, Operation::Matrix(res_m));
                },
            },
            MirValue::TraceAccess(_) => { },
            MirValue::PeriodicColumn(_) => { },
            MirValue::PublicInput(_) => { },
            MirValue::RandomValue(_) => { },
            MirValue::TraceAccessBinding(trace_access_binding) => {
                // Create Trace Access based on this binding
                let mut vec = vec![];
                for index in 0..trace_access_binding.size {
                    let val = graph.insert_op_value(SpannedMirValue {
                        span: spanned_mir_value.span.clone(),
                        value: MirValue::TraceAccess(
                            TraceAccess {
                                segment: trace_access_binding.segment,
                                column: trace_access_binding.offset + index,
                                row_offset: 0,  // ???
                            }
                        ),
                    });
                    vec.push(val);
                }
                graph.update_node(&node_index, Operation::Vector(vec));
            },
            MirValue::RandomValueBinding(random_value_binding) => {
                let mut vec = vec![];
                for index in 0..random_value_binding.size {
                    let val = graph.insert_op_value(SpannedMirValue {
                        span: spanned_mir_value.span.clone(),
                        value: MirValue::RandomValue(random_value_binding.offset + index),
                    });
                    vec.push(val);
                }
                graph.update_node(&node_index, Operation::Vector(vec));
            },
            MirValue::Vector(vec) => {
                let mut new_vec = vec![];
                for mir_value in vec {
                    let val = graph.insert_op_value(SpannedMirValue {
                        span: spanned_mir_value.span.clone(),
                        value: mir_value,
                    });
                    new_vec.push(val);
                }
                graph.update_node(&node_index, Operation::Vector(new_vec));
            },
            MirValue::Matrix(matrix) => {
                let mut new_matrix = vec![];
                for row in matrix {
                    let mut new_row = vec![];
                    for mir_value in row {
                        let val = graph.insert_op_value(SpannedMirValue {
                            span: spanned_mir_value.span.clone(),
                            value: mir_value,
                        });
                        new_row.push(val);
                    }
                    new_matrix.push(new_row);
                }
                graph.update_node(&node_index, Operation::Matrix(new_matrix));
            },
            MirValue::Variable(_mir_type, _, _node_index) => todo!(),
            MirValue::Definition(_vec, _node_index, _node_index1) => todo!(),
        }
    }

    fn visit_binary_op(&mut self, graph: &mut MirGraph, node_index: NodeIndex, lhs: NodeIndex, rhs: NodeIndex, binary_op: BinaryOp) {
        let lhs_op = graph.node(&lhs).op().clone();
        let rhs_op = graph.node(&rhs).op().clone();

        match (lhs_op, rhs_op) {
            (Operation::Value(SpannedMirValue { span: _, value: lhs_value }), Operation::Value(SpannedMirValue { span: _, value: rhs_value })) => {
                // Check value types to ensure scalar, raise diag otherwise
            },
            (Operation::Vector(lhs_vec), Operation::Vector(rhs_vec)) => {
                if lhs_vec.len() != rhs_vec.len() {
                    // Raise diag
                } else {
                    let mut new_vec = vec![];
                    for (lhs, rhs) in lhs_vec.iter().zip(rhs_vec.iter()) {
                        let new_node_index = match binary_op {
                            BinaryOp::Add => graph.insert_op_add(*lhs, *rhs),
                            BinaryOp::Sub => graph.insert_op_sub(*lhs, *rhs),
                            BinaryOp::Mul => graph.insert_op_mul(*lhs, *rhs),
                        };
                        new_vec.push(new_node_index);
                    }
                    graph.update_node(&node_index, Operation::Vector(new_vec));
                }
            },
            _ => { }
        }
    }

    fn visit_enf(&mut self, graph: &mut MirGraph, node_index: NodeIndex, child_node_index: NodeIndex) {
        let child_op = graph.node(&child_node_index).op().clone();

        match child_op {
            Operation::Value(SpannedMirValue { span: _, value: child_value }) => {
                // Check value types to ensure scalar, raise diag otherwise
            },
            Operation::Vector(child_vec) => {
                let mut new_vec = vec![];
                for child in child_vec.iter() {
                    let new_node_index = graph.insert_op_enf(*child);
                    new_vec.push(new_node_index);
                }
                graph.update_node(&node_index, Operation::Vector(new_vec));
            },
            _ => unreachable!()
        }
    }

    fn visit_fold(&mut self, graph: &mut MirGraph, node_index: NodeIndex, iterator: NodeIndex, fold_operator: FoldOperator, accumulator: NodeIndex) {
        // We need to expand this Fold into a nested sequence of binary expressions (add or mul depending on fold_operator)

        let iterator = graph.node(&iterator).op().clone();
        let iterator_node_indexes= match iterator {
            Operation::Vector(vec) => {
                vec
            },
            _ => unreachable!()
        };

        let mut acc_node_index = accumulator;

        match fold_operator {
            FoldOperator::Add => {
                for iterator_node_index in iterator_node_indexes {
                    let new_acc_node_index = graph.insert_op_add(acc_node_index, iterator_node_index);
                    acc_node_index = new_acc_node_index;
                }
            },
            FoldOperator::Mul => {
                for iterator_node_index in iterator_node_indexes {
                    let new_acc_node_index = graph.insert_op_mul(acc_node_index, iterator_node_index);
                    acc_node_index = new_acc_node_index;
                }
            },
        }

        // Finally, replace the Fold with the expanded expression
        graph.update_node(&node_index, graph.node(&acc_node_index).op().clone());
    }

    fn visit_variable(&mut self, _graph: &mut MirGraph, _node_index: NodeIndex, spanned_variable: SpannedVariable) {
        // Just check that the variable is a scalar, raise diag otherwise
        // List comprehension bodies should only be scalar expressions
        match spanned_variable.ty {
            MirType::Felt => { },
            MirType::Vector(_size) => unreachable!(),
            MirType::Matrix(_rows, _cols) => unreachable!(),
            MirType::Definition(_vec, _) => todo!(),
        }
    }

    fn visit_if(&mut self, graph: &mut MirGraph, node_index: NodeIndex, cond_node_index: NodeIndex, then_node_index: NodeIndex, else_node_index: NodeIndex) {
        let cond_op = graph.node(&cond_node_index).op().clone();
        let then_op = graph.node(&then_node_index).op().clone();
        let else_op = graph.node(&else_node_index).op().clone();

        match (cond_op, then_op, else_op) {
            (
                Operation::Value(SpannedMirValue { span: _, value: cond_value }),
                Operation::Value(SpannedMirValue { span: _, value: then_value }),
                Operation::Value(SpannedMirValue { span: _, value: else_value }),                
            ) => {
                // Check value types to ensure scalar, raise diag otherwise
            },
            (
                Operation::Vector(cond_vec),
                Operation::Vector(then_vec),
                Operation::Vector(else_vec),
             ) => {
                if cond_vec.len() != then_vec.len() || cond_vec.len() != else_vec.len() {
                    // Raise diag
                } else {
                    let mut new_vec = vec![];
                    for ((cond, then), else_) in cond_vec.iter().zip(then_vec.iter()).zip(else_vec.iter()) {
                        let new_node_index = graph.insert_op_if(*cond, *then, *else_);
                        new_vec.push(new_node_index);
                    }
                    graph.update_node(&node_index, Operation::Vector(new_vec));
                }
            },
            _ => unreachable!()
        }
    }

    fn visit_boundary(&mut self, graph: &mut MirGraph, node_index: NodeIndex, boundary: Boundary, child_node_index: NodeIndex) {
        let child_op = graph.node(&child_node_index).op().clone();

        match child_op {
            Operation::Value(SpannedMirValue { span: _, value: child_value }) => {
                // Check value types to ensure scalar, raise diag otherwise
            },
            Operation::Vector(child_vec) => {
                let mut new_vec = vec![];
                for child in child_vec.iter() {
                    let new_node_index = graph.insert_op_boundary(boundary, *child);
                    new_vec.push(new_node_index);
                }
                graph.update_node(&node_index, Operation::Vector(new_vec));
            },
            _ => unreachable!()
        }
    }

    fn visit_for(&mut self, graph: &mut MirGraph, node_index: NodeIndex, iterators: Vec<NodeIndex>, body: NodeIndex, selector: Option<NodeIndex>) {
        
        // For each value produced by the iterators, we need to:
        // - Duplicate the body
        // - Visit the body and replace the Variables with the value (with the correct index depending on the binding)
        // If there is a selector, we need to enforce the selector on the body through an if node ?

        // Check iterator lengths
        if iterators.is_empty() {
            unreachable!(); // Raise diag
        }
        let iterator_expected_len = match graph.node(&iterators[0]).op().clone() {
            Operation::Vector(vec) => vec.len(),
            _ => unreachable!(),
        };
        for iterator in iterators.iter().skip(1) {
            match graph.node(&iterator).op().clone() {
                Operation::Vector(vec) => {
                    if vec.len() != iterator_expected_len {
                        unreachable!(); // Raise diag
                    }
                },
                _ => unreachable!(),
            }
        }

        let iterator_nodes_indices = iterators.iter().map(|iterator| {
            let iterator_op = graph.node(iterator).op().clone();
            match iterator_op {
                Operation::Vector(vec) => vec,
                _ => unreachable!(),
            }
        }).collect::<Vec<_>>();

        let mut new_vec = vec![];
        for i in 0..iterator_expected_len {
            let new_node_index = graph.insert_op_placeholder();
            new_vec.push(new_node_index);

            let iterators_i = iterator_nodes_indices.iter().map(|vec| vec[i]).collect::<Vec<_>>();

            self.bodies_to_inline.insert(new_node_index, 
                ForInliningContext {
                    body_index: body,
                    iterators: iterators_i,
                    selector: selector,
                    index: i,
                    parent_for: node_index,
                }
            );
        }

        graph.update_node(&node_index, Operation::Vector(new_vec));
    }

    fn visit_first_pass(&mut self, graph: &mut MirGraph, node_index: NodeIndex) {
        let op = graph.node(&node_index).op().clone();
        match op {
            Operation::Value(spanned_mir_value) => {
                // Transform values to scalar nodes (in the case of a vector or matrix, transform into Operation::Vector or Operation::Matrix)
                self.visit_value(graph, node_index, spanned_mir_value);
            },
            Operation::Add(lhs, rhs) => {
                self.visit_binary_op(graph, node_index, lhs, rhs, BinaryOp::Add);
            },
            Operation::Sub(lhs, rhs) => {
                self.visit_binary_op(graph, node_index, lhs, rhs, BinaryOp::Sub);
            },
            Operation::Mul(lhs, rhs) => {
                self.visit_binary_op(graph, node_index, lhs, rhs, BinaryOp::Mul);
            },
            Operation::Enf(child_node_index) => {
                self.visit_enf(graph, node_index, child_node_index);
            },
            Operation::Fold(iterator, fold_operator, accumulator) => {
                self.visit_fold(graph, node_index, iterator, fold_operator, accumulator);
            },
            Operation::For(iterators, body, selector) => {
                // For each value produced by the iterators, we need to:
                // - Duplicate the body
                // - Visit the body and replace the Variables with the value (with the correct index depending on the binding)
                // We then have a vector, that we can either fold up or enforce on each value

                self.visit_for(graph, node_index, iterators, body, selector);
            },
            Operation::If(cond_node_index, then_node_index, else_node_index) => {
                self.visit_if(graph, node_index, cond_node_index, then_node_index, else_node_index);
            },

            Operation::Boundary(boundary, child_node_index) => {
                self.visit_boundary(graph, node_index, boundary, child_node_index);
            },

            Operation::Variable(spanned_variable) => {
                self.visit_variable(graph, node_index, spanned_variable);
            },

            // These are already unrolled
            Operation::Vector(_vec) => { }, 
            Operation::Matrix(_vec) => { },

            // These should not exist / be accessible from roots after inlining
            Operation::Placeholder => { },
            Operation::Definition(_vec, _node_index, _vec1) => { },
            Operation::Call(_node_index, _vec) => { },
        }
    }

    fn visit_second_pass(&mut self, graph: &mut MirGraph, node_index: NodeIndex) {
        if self.bodies_to_inline.contains_key(&node_index) {
            // A new body to inline, we should replace the op with the corresponding iteration in the body
            self.for_inlining_context = self.bodies_to_inline.get(&node_index).unwrap().clone();
            self.nodes_to_replace.clear();
            self.visit_later(self.for_inlining_context.body_index);
        } else {
            // Normal visit, insert in the graph the same instruction
            let op = graph.node(&node_index).op().clone();
            match op {
                Operation::Variable(spanned_variable) => { 
                    self.nodes_to_replace.insert(node_index, self.for_inlining_context.iterators[spanned_variable.argument_position]);
                },
                Operation::Value(v) => { },
                Operation::Add(lhs, rhs) => {
                    let new_lhs_node_index = self.nodes_to_replace.get(&lhs).unwrap_or(&lhs).clone();
                    let new_rhs_node_index = self.nodes_to_replace.get(&rhs).unwrap_or(&rhs).clone();

                    let new_node_index = graph.insert_op_add(new_lhs_node_index, new_rhs_node_index);
                    self.nodes_to_replace.insert(node_index, new_node_index);
                },
                Operation::Sub(lhs, rhs) => {
                    let new_lhs_node_index = self.nodes_to_replace.get(&lhs).unwrap_or(&lhs).clone();
                    let new_rhs_node_index = self.nodes_to_replace.get(&rhs).unwrap_or(&rhs).clone();

                    let new_node_index = graph.insert_op_sub(new_lhs_node_index, new_rhs_node_index);
                    self.nodes_to_replace.insert(node_index, new_node_index);
                },
                Operation::Mul(lhs, rhs) => {
                    let new_lhs_node_index = self.nodes_to_replace.get(&lhs).unwrap_or(&lhs).clone();
                    let new_rhs_node_index = self.nodes_to_replace.get(&rhs).unwrap_or(&rhs).clone();

                    let new_node_index = graph.insert_op_mul(new_lhs_node_index, new_rhs_node_index);
                    self.nodes_to_replace.insert(node_index, new_node_index);
                },
                Operation::Fold(iter, f_op, acc ) => {
                    let new_iter = self.nodes_to_replace.get(&iter).unwrap_or(&iter).clone();
                    let new_acc = self.nodes_to_replace.get(&acc).unwrap_or(&acc).clone();

                    let new_node_index = graph.insert_op_fold(new_iter, f_op, new_acc);
                    self.nodes_to_replace.insert(node_index, new_node_index);
                },
                Operation::If(cond, then, else_) => { 
                    let new_cond = self.nodes_to_replace.get(&cond).unwrap_or(&cond).clone();
                    let new_then = self.nodes_to_replace.get(&then).unwrap_or(&then).clone();
                    let new_else = self.nodes_to_replace.get(&else_).unwrap_or(&else_).clone();

                    let new_node_index = graph.insert_op_if(new_cond, new_then, new_else);
                    self.nodes_to_replace.insert(node_index, new_node_index);
                },
                Operation::Boundary(b, b_node_index) => { 
                    let new_b_node_index = self.nodes_to_replace.get(&b_node_index).unwrap_or(&b_node_index).clone();
                    let new_node_index = graph.insert_op_boundary(b, new_b_node_index);
                    self.nodes_to_replace.insert(node_index, new_node_index);
                },
                Operation::Vector(v) => { 
                    let new_v = v.iter().map(|node_index| {
                        self.nodes_to_replace.get(node_index).unwrap_or(node_index).clone()
                    }).collect();
                    let new_node_index = graph.insert_op_vector(new_v);
                    self.nodes_to_replace.insert(node_index, new_node_index);
                },
                Operation::Matrix(m) => { 
                    let new_m = m.iter().map(|row| {
                        row.iter().map(|node_index| {
                            self.nodes_to_replace.get(node_index).unwrap_or(node_index).clone()
                        }).collect()
                    }).collect();
                    let new_node_index = graph.insert_op_matrix(new_m);
                    self.nodes_to_replace.insert(node_index, new_node_index);
                },

                Operation::Placeholder => unreachable!(),
                Operation::Enf(_) => unreachable!(),
                Operation::Definition(_, _, _) =>  unreachable!(),
                Operation::Call(_, _) =>  unreachable!(),
                Operation::For(_, _, _) => unreachable!(),
            }

            if node_index == self.for_inlining_context.body_index {
                // We have finished inlining the body, we can now replace the node_index in the current index of the parent For
                let new_node_index = self.nodes_to_replace.get(&node_index).unwrap_or(&node_index).clone();

                let parent_for = self.for_inlining_context.parent_for;
                let parent_for_op = graph.node(&parent_for).op().clone();
                match parent_for_op {
                    Operation::Vector(vec) => {
                        let mut new_vec = vec.clone();

                        let new_node_to_update_at_index = if let Some(selector) = self.for_inlining_context.selector {
                            let zero_node = graph.insert_op_value(SpannedMirValue {
                                span: Default::default(),
                                value: MirValue::Constant(ConstantValue::Felt(0)),
                            });
                            let if_node = graph.insert_op_if(selector, new_node_index, zero_node);
                            if_node
                        } else {
                            new_node_index
                        };

                        new_vec[self.for_inlining_context.index] = new_node_to_update_at_index;

                        graph.update_node(&parent_for, Operation::Vector(new_vec));
                    },
                    _ => unreachable!(),
                }
            }
        }

    }
}

impl VisitContext for Unrolling {
    fn visit(&mut self, graph: &mut MirGraph, node_index: NodeIndex) {
        if self.during_first_pass {
            self.visit_first_pass(graph, node_index);
        } else {
            self.visit_second_pass(graph, node_index);
        }
    }

    fn as_stack_mut(&mut self) -> &mut Vec<NodeIndex> {
        &mut self.work_stack
    }
    
    type Graph = MirGraph;
    
    fn boundary_roots(&self, graph: &Self::Graph) -> HashSet<NodeIndex> {
        if self.during_first_pass {
            return graph.boundary_constraints_roots.clone();
        } else {
            return self.bodies_to_inline.keys().cloned().collect();
        }
    }
    
    fn integrity_roots(&self, graph: &Self::Graph) -> HashSet<NodeIndex> {
        return graph.integrity_constraints_roots.clone()
    }
    
    fn visit_order(&self) -> super::VisitOrder {
        if self.during_first_pass {
            return super::VisitOrder::PostOrder;
        } else {
            return super::VisitOrder::PostOrder;
        }
    }
}
