use std::ops::ControlFlow;

use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{graph::pretty, MirGraph, Node, NodeIndex, Operation, SpannedVariable};

//pub struct Inlining<'a> {
//     #[allow(unused)]
//     diagnostics: &'a DiagnosticsHandler,
//}

pub struct Inlining {}

//impl<'p> Pass for Inlining<'p> {}
impl Pass for Inlining {
    type Input<'a> = MirGraph;
    type Output<'a> = MirGraph;
    type Error = ();

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        match self.run_visitor(&mut ir) {
            ControlFlow::Continue(()) => Ok(ir),
            ControlFlow::Break(err) => Err(err),
        }
    }
}

// impl<'a> Inlining<'a> {
//     pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
//         Self { diagnostics }
//         Self {}
//     }
// }
impl Inlining {
    pub fn new() -> Self {
        Self {}
    }
    //TODO MIR: Implement inlining pass on MIR
    // 1. Understand the basics of the previous inlining process
    // 2. Remove what is done during lowering from AST to MIR (unroll, ...)
    // 3. Check how it translates to the MIR structure
    fn run_visitor(&mut self, ir: &mut MirGraph) -> ControlFlow<()> {
        inline_all(ir);
        ControlFlow::Continue(())
    }
}

fn inline_all(ir: &mut MirGraph) {
    for node_uindex in 0..ir.num_nodes() {
        let def_node_index = NodeIndex(node_uindex);
        let def_node = ir.node(&def_node_index).clone();
        if let Operation::Definition(_, _, body) = &def_node.op {
            println!("{}: Def node: {:?}", def_node_index.0, def_node);
            for (index_in_body, call_index) in body.iter().enumerate() {
                inline_call(ir, call_index, &def_node_index, index_in_body);
            }
        }
    }
}

fn inline_call(
    ir: &mut MirGraph,
    call_index: &NodeIndex,
    outer_def_index: &NodeIndex,
    index_in_body: usize,
) {
    let call_node = ir.node(call_index).clone();
    if let Operation::Call(def_index, arg_indexes) = &call_node.op {
        println!("Call: {:?} {:?}", def_index, arg_indexes);
        let new_nodes = inline_body(ir, def_index);
        println!("new_nodes: {:?}", new_nodes);
        let def_node = ir.node(outer_def_index);
        if let Operation::Definition(outer_func_arg_indexes, outer_func_ret, outer_body) =
            &def_node.op
        {
            println!(
                "args: {:?} ret: {:?} body: {:?}",
                outer_func_arg_indexes, outer_func_ret, outer_body
            );
            let mut new_body = outer_body.clone();
            new_body[index_in_body] = *new_nodes.last().unwrap();
            for op_idx in new_nodes.iter().rev().skip(1) {
                new_body.insert(index_in_body, *op_idx);
            }
            ir.update_node(
                outer_def_index,
                Operation::Definition(outer_func_arg_indexes.clone(), *outer_func_ret, new_body),
            );
            //inline_swap_args(ir, outer_func_arg_indexes, arg_indexes);
        }
    }
}

fn inline_body(ir: &mut MirGraph, def_index: &NodeIndex) -> Vec<NodeIndex> {
    let def_node = ir.node(def_index).clone();
    let mut new_nodes = vec![];
    if let Operation::Definition(_, _, body) = &def_node.op {
        for node_index in body {
            new_nodes.push(inline_op(ir, node_index));
        }
    }
    new_nodes
}

fn inline_op(ir: &mut MirGraph, op_index: &NodeIndex) -> NodeIndex {
    let op_node = ir.node(op_index).clone();
    let new_node = ir.insert_placeholder_op();
    let op = op_node.op.clone();
    ir.update_node(&new_node, op.clone());
    new_node
}

fn inline_swap_args(
    ir: &mut MirGraph,
    outer_func_arg_indexes: &Vec<NodeIndex>,
    call_arg_indexes: &Vec<NodeIndex>,
) {
    for (outer_func_arg_index, call_arg_index) in
        outer_func_arg_indexes.iter().zip(call_arg_indexes)
    {
        println!(
            "outer_func_arg_index: {:?} call_arg_index: {:?}",
            outer_func_arg_index, call_arg_index
        );
    }
}
