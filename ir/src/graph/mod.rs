use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use miden_diagnostics::SourceSpan;

use crate::ir::*;

/// A unique identifier for a node in an [AlgebraicGraph]
///
/// The raw value of this identifier is an index in the `nodes` vector
/// of the [AlgebraicGraph] struct.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeIndex(pub usize);
impl core::ops::Add<usize> for NodeIndex {
    type Output = NodeIndex;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl core::ops::Add<usize> for &NodeIndex {
    type Output = NodeIndex;

    fn add(self, rhs: usize) -> Self::Output {
        NodeIndex(self.0 + rhs)
    }
}

/// A node in the [AlgebraicGraph]
#[derive(Debug, Clone)]
pub struct Node {
    /// The operation represented by this node
    pub op: Operation,
}
impl Node {
    /// Get the underlying [Operation] represented by this node
    #[inline]
    pub const fn op(&self) -> &Operation {
        &self.op
    }
}

/// The MirGraph is a directed acyclic graph used to represent integrity constraints. To
/// store it compactly, it is represented as a vector of nodes where each node references other
/// nodes by their index in the vector.
///
/// Within the graph, constraint expressions can overlap and share subgraphs, since new expressions
/// reuse matching existing nodes when they are added, rather than creating new nodes.
///
/// - Leaf nodes (with no outgoing edges) are constants or references to trace cells (i.e. column 0
///   in the current row or column 5 in the next row).
/// - Tip nodes with no incoming edges (no parent nodes) always represent constraints, although they
///   do not necessarily represent all constraints. There could be constraints which are also
///   subgraphs of other constraints.
#[derive(Default, Debug, Clone)]
pub struct MirGraph {
    /// All nodes in the graph.
    nodes: Vec<Node>,
    use_list: HashMap<NodeIndex, Vec<NodeIndex>>,
    pub functions: BTreeMap<QualifiedIdentifier, NodeIndex>,
    pub evaluators: BTreeMap<QualifiedIdentifier, NodeIndex>,
}

impl MirGraph {
    /// Creates a new graph from a list of nodes.
    pub fn new(nodes: Vec<Node>) -> Self {
        Self {
            nodes,
            use_list: HashMap::default(),
            functions: BTreeMap::new(),
            evaluators: BTreeMap::new(),
        }
    }

    /// Returns the node with the specified index.
    pub fn node(&self, index: &NodeIndex) -> &Node {
        &self.nodes[index.0]
    }

    pub fn update_node(&mut self, index: &NodeIndex, op: Operation) {
        if let Some(node) = self.nodes.get(index.0) {
            let prev_op = node.op().clone();
            let prev_children_nodes = get_children(prev_op);

            for child in prev_children_nodes {
                self.remove_use(child, *index);
            }

            let children_nodes = get_children(op.clone());

            for child in children_nodes {
                self.add_use(child, *index);
            }
        }

        if let Some(node) = self.nodes.get_mut(index.0) {
            *node = Node { op };
        }
    }

    pub fn add_use(&mut self, node_index: NodeIndex, use_index: NodeIndex) {
        self.use_list.entry(node_index).or_default().push(use_index);
    }

    pub fn remove_use(&mut self, node_index: NodeIndex, use_index: NodeIndex) {
        self.use_list
            .entry(node_index)
            .and_modify(|vec| vec.retain(|&index| index != use_index));
    }

    /// Returns the number of nodes in the graph.
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    // TODO : Instead of checking the all tree recursively, maybe we should:
    // - Check each node when adding it to the graph (depending on its children)
    // - Check the modified nodes when applying a pass (just to the edited ops, not the whole graph)
    /*pub fn check_typing_rules(&self, node_index: NodeIndex) -> Result<(), CompileError> {
        // Todo: implement the typing rules
        // Propagate types recursively through the graph and check that the types are consistent?
        match self.node(&node_index).op() {
            Operation::Value(_val) => Ok(()),
            Operation::Add(_lhs, _rhs) => todo!(),
            /*{
                let lhs_node = self.node(lhs);
                let rhs_node = self.node(rhs);
                if lhs_node.ty() != rhs_node.ty() {
                    Err(())
                } else {
                    Ok(())
                }
            },*/
            Operation::Sub(_lhs, _rhs) => todo!(),
            Operation::Mul(_lhs, _rhs) => todo!(),
            Operation::Enf(_node_index) => todo!(),
            Operation::Call(_func_def, _args) => todo!(),
            Operation::Fold(_iterator, _fold_operator, _accumulator) => todo!(),
            Operation::For(_iterator, _body, _selector) => todo!(),
            Operation::If(_condition, _then, _else) => todo!(),
            Operation::Variable(_var) => todo!(),
            Operation::Definition(_params, _return, _body) => todo!(),
            Operation::Vector(_vec) => todo!(),
            Operation::Matrix(_vec) => todo!(),
        }
    }*/

    /// Insert the operation and return its node index. If an identical node already exists, return
    /// that index instead.
    pub(crate) fn insert_node(&mut self, op: Operation) -> NodeIndex {
        let children_nodes = get_children(op.clone());

        let node_index = self.nodes.iter().position(|n| *n.op() == op).map_or_else(
            || {
                // create a new node.
                let index = self.nodes.len();
                self.nodes.push(Node { op });
                NodeIndex(index)
            },
            |index| {
                // return the existing node's index.
                NodeIndex(index)
            },
        );

        for child in children_nodes {
            self.add_use(child, node_index);
        }

        node_index
    }

    /// Insert a placeholder operation and return its node index. This will create duplicate nodes if called multiple times.
    pub fn insert_placeholder_op(&mut self) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(Node {
            op: Operation::Value(SpannedMirValue {
                span: SourceSpan::default(),
                value: MirValue::Constant(ConstantValue::Felt(0)),
            }),
        });
        NodeIndex(index)
    }
}

#[derive(Debug, Clone)]
struct PrettyCounters {
    pub var_count: usize,
    pub fn_count: usize,
}

#[derive(Clone)]
struct PrettyCtx<'a> {
    pub graph: &'a MirGraph,
    pub indent: usize,
    pub nl: &'a str,
    pub in_block: bool,
    pub counters: Rc<RefCell<PrettyCounters>>,
}

impl<'a> PrettyCtx<'a> {
    fn new(graph: &'a MirGraph) -> Self {
        let counters = Rc::new(RefCell::new(PrettyCounters {
            var_count: 0,
            fn_count: 0,
        }));
        Self {
            graph,
            indent: 0,
            nl: "\n",
            in_block: false,
            counters,
        }
    }

    fn add_indent(&self, indent: usize) -> Self {
        Self {
            indent: self.indent + indent,
            ..self.clone()
        }
    }

    fn with_indent(&self, indent: usize) -> Self {
        Self {
            indent,
            ..self.clone()
        }
    }

    fn increment_var_count(&self) -> Self {
        self.counters.borrow_mut().var_count += 1;
        self.clone()
    }

    fn increment_fn_count(&self) -> Self {
        self.counters.borrow_mut().fn_count += 1;
        self.clone()
    }

    fn with_nl(&self, nl: &'a str) -> Self {
        Self { nl, ..self.clone() }
    }

    fn with_in_block(&self, in_block: bool) -> Self {
        Self {
            in_block,
            ..self.clone()
        }
    }

    fn indent_str(&self) -> String {
        if self.nl == "\n" {
            "  ".repeat(self.indent)
        } else {
            "".to_string()
        }
    }
}

impl std::fmt::Debug for PrettyCtx<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrettyCtx")
            .field("indent", &self.indent)
            .field("nl", &self.nl)
            .field("in_block", &self.in_block)
            .field("var_count", &self.counters.borrow().var_count)
            .field("fn_count", &self.counters.borrow().fn_count)
            .finish()
    }
}

pub fn pretty(graph: &MirGraph, roots: &[NodeIndex]) -> String {
    let mut result = String::from("\n\n");
    let mut ctx = PrettyCtx::new(graph);
    for root in roots {
        pretty_rec(*root, &mut ctx, &mut result);
    }
    result
}

fn pretty_rec(node: NodeIndex, ctx: &mut PrettyCtx, result: &mut String) {
    let node = ctx.graph.node(&node);
    let op = node.op();
    match op {
        Operation::Definition(args_idx, ret_idx, body_idx) => {
            result.push_str(&format!(
                "{}fn f{}(",
                ctx.indent_str(),
                ctx.counters.borrow().fn_count
            ));
            ctx.increment_fn_count();
            for (i, arg) in args_idx.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }
                pretty_rec(*arg, &mut ctx.with_indent(0).with_nl(""), result);
            }
            result.push_str(") -> ");
            match ret_idx {
                Some(ret_idx) => pretty_rec(*ret_idx, &mut ctx.with_indent(0).with_nl(""), result),
                None => result.push_str("()"),
            }
            result.push_str("{\n");
            for op_idx in body_idx {
                pretty_rec(*op_idx, &mut ctx.add_indent(1).with_in_block(true), result);
            }
            result.push_str(&format!(
                "{}return x{};\n",
                ctx.add_indent(1).indent_str(),
                ctx.counters.borrow().var_count
            ));
            result.push_str(&format!("{}}}\n", ctx.indent_str()));
        }
        Operation::Value(spanned_val) => {
            let val = &spanned_val.value;
            match val {
                MirValue::Variable(ty, pos, func) => {
                    if ctx.in_block {
                        result.push_str(&format!("x{}", pos));
                    } else {
                        result.push_str(&format!(
                            "{}x{}: {:?}{}",
                            ctx.indent_str(),
                            pos,
                            ty,
                            ctx.nl
                        ));
                    }
                }
                val => result.push_str(&format!("{}{:?}{}", ctx.indent_str(), val, ctx.nl)),
            };
        }
        Operation::Add(lhs, rhs) => {
            pretty_ssa((lhs, rhs), ctx, "+", result);
        }
        op => result.push_str(&format!("{}{:?}\n", ctx.indent_str(), op)),
    }
}

fn pretty_ssa(
    (lhs, rhs): (&NodeIndex, &NodeIndex),
    ctx: &mut PrettyCtx,
    op_str: &str,
    result: &mut String,
) {
    result.push_str(&ctx.indent_str());
    ctx.increment_var_count();
    result.push_str(&format!("let x{} = ", ctx.counters.borrow().var_count));
    pretty_rec(*lhs, &mut ctx.add_indent(1).with_nl(""), result);
    result.push_str(&format!(" {} ", op_str));
    pretty_rec(*rhs, &mut ctx.add_indent(1).with_nl(""), result);
    result.push_str(&format!(";\n{}", if ctx.in_block { "" } else { ctx.nl }));
}

fn get_children(op: Operation) -> Vec<NodeIndex> {
    match op {
        Operation::Value(_spanned_mir_value) => vec![],
        Operation::Add(lhs, rhs) => vec![lhs, rhs],
        Operation::Sub(lhs, rhs) => vec![lhs, rhs],
        Operation::Mul(lhs, rhs) => vec![lhs, rhs],
        Operation::Enf(child_index) => vec![child_index],
        Operation::Call(def, args) => {
            let mut ret = args;
            ret.push(def);
            ret
        }
        Operation::Fold(iterator_index, _fold_operator, accumulator_index) => {
            vec![iterator_index, accumulator_index]
        }
        Operation::For(iterators, body_index, selector_index) => {
            let mut ret = iterators;
            ret.push(body_index);
            if let Some(selector_index) = selector_index {
                ret.push(selector_index);
            }
            ret
        }
        Operation::If(condition_index, then_index, else_index) => {
            vec![condition_index, then_index, else_index]
        }
        Operation::Variable(_spanned_variable) => vec![],
        Operation::Definition(params, return_index, body) => {
            let mut ret = params;
            ret.extend_from_slice(&body);
            if let Some(return_index) = return_index {
                ret.push(return_index);
            }
            ret
        }
        Operation::Vector(vec) => vec,
        Operation::Matrix(vec) => vec.iter().flatten().copied().collect(),
        Operation::Boundary(_boundary, child_index) => vec![child_index],
    }
}
