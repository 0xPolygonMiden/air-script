use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;
// use miden_diagnostics::SourceSpan;

use crate::ir::*;
use crate::passes::Graph;

/// A unique identifier for a node in an [AlgebraicGraph]
///
/// The raw value of this identifier is an index in the `nodes` vector
/// of the [AlgebraicGraph] struct.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    pub boundary_constraints_roots: HashSet<NodeIndex>,
    pub integrity_constraints_roots: HashSet<NodeIndex>,
}

/// Helpers for inserting operations
impl MirGraph {
    pub fn insert_op_value(&mut self, value: SpannedMirValue) -> NodeIndex {
        self.insert_node(Operation::Value(value))
    }

    pub fn insert_op_add(&mut self, lhs: NodeIndex, rhs: NodeIndex) -> NodeIndex {
        self.insert_node(Operation::Add(lhs, rhs))
    }

    pub fn insert_op_sub(&mut self, lhs: NodeIndex, rhs: NodeIndex) -> NodeIndex {
        self.insert_node(Operation::Sub(lhs, rhs))
    }

    pub fn insert_op_mul(&mut self, lhs: NodeIndex, rhs: NodeIndex) -> NodeIndex {
        self.insert_node(Operation::Mul(lhs, rhs))
    }

    pub fn insert_op_enf(&mut self, node_index: NodeIndex) -> NodeIndex {
        self.insert_node(Operation::Enf(node_index))
    }

    pub fn insert_op_call(&mut self, def: NodeIndex, args: Vec<NodeIndex>) -> NodeIndex {
        self.insert_node(Operation::Call(def, args))
    }

    pub fn insert_op_fold(
        &mut self,
        iterator: NodeIndex,
        fold_operator: FoldOperator,
        accumulator: NodeIndex,
    ) -> NodeIndex {
        self.insert_node(Operation::Fold(iterator, fold_operator, accumulator))
    }

    pub fn insert_op_for(
        &mut self,
        iterators: Vec<NodeIndex>,
        body: NodeIndex,
        selector: Option<NodeIndex>,
    ) -> NodeIndex {
        self.insert_node(Operation::For(iterators, body, selector))
    }

    pub fn insert_op_if(
        &mut self,
        condition: NodeIndex,
        then: NodeIndex,
        else_: NodeIndex,
    ) -> NodeIndex {
        self.insert_node(Operation::If(condition, then, else_))
    }

    pub fn insert_op_variable(&mut self, variable: SpannedVariable) -> NodeIndex {
        self.insert_node(Operation::Variable(variable))
    }

    pub fn insert_op_definition(
        &mut self,
        params: Vec<NodeIndex>,
        return_: Option<NodeIndex>,
        body: Vec<NodeIndex>,
    ) -> NodeIndex {
        self.insert_node(Operation::Definition(params, return_, body))
    }

    pub fn insert_op_vector(&mut self, vec: Vec<NodeIndex>) -> NodeIndex {
        self.insert_node(Operation::Vector(vec))
    }

    pub fn insert_op_matrix(&mut self, vec: Vec<Vec<NodeIndex>>) -> NodeIndex {
        self.insert_node(Operation::Matrix(vec))
    }

    pub fn insert_op_boundary(&mut self, boundary: Boundary, child: NodeIndex) -> NodeIndex {
        self.insert_node(Operation::Boundary(boundary, child))
    }

    pub fn insert_op_placeholder(&mut self) -> NodeIndex {
        self.insert_placeholder_op()
    }
}

impl MirGraph {
    /// Creates a new graph from a list of nodes.
    pub fn new(nodes: Vec<Node>) -> Self {
        Self {
            nodes,
            use_list: HashMap::default(),
            functions: BTreeMap::new(),
            evaluators: BTreeMap::new(),
            boundary_constraints_roots: HashSet::new(),
            integrity_constraints_roots: HashSet::new(),
        }
    }

    /// Returns the node with the specified index.
    pub fn node(&self, index: &NodeIndex) -> &Node {
        &self.nodes[index.0]
    }

    pub fn insert_boundary_constraints_root(&mut self, index: NodeIndex) {
        self.boundary_constraints_roots.insert(index);
    }

    pub fn remove_boundary_constraints_root(&mut self, index: NodeIndex) {
        self.boundary_constraints_roots.remove(&index);
    }

    pub fn insert_integrity_constraints_root(&mut self, index: NodeIndex) {
        self.integrity_constraints_roots.insert(index);
    }

    pub fn remove_integrity_constraints_root(&mut self, index: NodeIndex) {
        self.integrity_constraints_roots.remove(&index);
    }

    pub fn update_node(&mut self, index: &NodeIndex, op: Operation) {
        if let Some(node) = self.nodes.get(index.0) {
            let prev_op = node.op().clone();
            let prev_children_nodes = self.children(&prev_op);

            for child in prev_children_nodes {
                self.remove_use(child, *index);
            }

            let children_nodes = self.children(&op);

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
    fn insert_node(&mut self, op: Operation) -> NodeIndex {
        let children_nodes = self.children(&op);

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
    fn insert_placeholder_op(&mut self) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(Node {
            op: Operation::Placeholder,
        });
        NodeIndex(index)
    }
}

impl Graph for MirGraph {
    fn node(&self, node_index: &NodeIndex) -> &Node {
        MirGraph::node(self, node_index)
    }
    fn children(&self, node: &Operation) -> Vec<NodeIndex> {
        match node {
            Operation::Value(_spanned_mir_value) => vec![],
            Operation::Add(lhs, rhs) => vec![*lhs, *rhs],
            Operation::Sub(lhs, rhs) => vec![*lhs, *rhs],
            Operation::Mul(lhs, rhs) => vec![*lhs, *rhs],
            Operation::Enf(child_index) => vec![*child_index],
            Operation::Call(def, args) => {
                let mut ret = args.clone();
                ret.push(*def);
                ret
            }
            Operation::Fold(iterator_index, _fold_operator, accumulator_index) => {
                vec![*iterator_index, *accumulator_index]
            }
            Operation::For(iterators, body_index, selector_index) => {
                let mut ret = iterators.clone();
                ret.push(*body_index);
                if let Some(selector_index) = selector_index {
                    ret.push(*selector_index);
                }
                ret
            }
            Operation::If(condition_index, then_index, else_index) => {
                vec![*condition_index, *then_index, *else_index]
            }
            Operation::Variable(_spanned_variable) => vec![],
            Operation::Definition(params, return_index, body) => {
                let mut ret = params.clone();
                ret.extend_from_slice(&body);
                if let Some(return_index) = return_index {
                    ret.push(*return_index);
                }
                ret
            }
            Operation::Vector(vec) => vec.clone(),
            Operation::Matrix(vec) => vec.iter().flatten().copied().collect(),
            Operation::Boundary(_boundary, child_index) => vec![*child_index],
            Operation::Placeholder => vec![],
        }
    }
}

#[derive(Debug, Clone)]
struct PrettyShared<'a> {
    pub var_count: usize,
    pub fn_count: usize,
    // BTreeMap from function index to function id
    pub fns: BTreeMap<usize, usize>,
    pub roots: &'a [NodeIndex],
}

#[derive(Clone)]
struct PrettyCtx<'a> {
    pub graph: &'a MirGraph,
    pub indent: usize,
    pub nl: &'a str,
    pub in_block: bool,
    pub show_var_names: bool,
    pub shared: Rc<RefCell<PrettyShared<'a>>>,
}

impl<'a> PrettyCtx<'a> {
    fn new(graph: &'a MirGraph, roots: &'a [NodeIndex]) -> Self {
        let shared = Rc::new(RefCell::new(PrettyShared {
            var_count: 0,
            fn_count: 0,
            fns: BTreeMap::new(),
            roots,
        }));
        Self {
            graph,
            indent: 0,
            nl: "\n",
            in_block: false,
            shared,
            show_var_names: true,
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
        self.shared.borrow_mut().var_count += 1;
        self.clone()
    }

    fn increment_fn_count(&self, node_idx: &NodeIndex) -> Self {
        let fn_count = self.shared.borrow().fn_count;
        self.shared.borrow_mut().fns.insert(node_idx.0, fn_count);
        self.shared.borrow_mut().fn_count += 1;
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

    fn show_var_names(&self, show_var_names: bool) -> Self {
        Self {
            show_var_names,
            ..self.clone()
        }
    }
}

impl std::fmt::Debug for PrettyCtx<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrettyCtx")
            .field("indent", &self.indent)
            .field("nl", &self.nl)
            .field("in_block", &self.in_block)
            .field("var_count", &self.shared.borrow().var_count)
            .field("fn_count", &self.shared.borrow().fn_count)
            .finish()
    }
}

pub fn pretty(graph: &MirGraph, roots: &[NodeIndex]) -> String {
    let mut result = String::from("");
    let mut ctx = PrettyCtx::new(graph, roots);
    for root in roots {
        pretty_rec(*root, &mut ctx, &mut result);
        ctx.shared.borrow_mut().var_count = 0; // reset var count for next function
    }
    result
}

fn pretty_rec(node_idx: NodeIndex, ctx: &mut PrettyCtx, result: &mut String) {
    let node = ctx.graph.node(&node_idx);
    let op = node.op();
    match op {
        Operation::Definition(args_idx, ret_idx, body_idx) => {
            result.push_str(&format!(
                "{}fn f{}(",
                ctx.indent_str(),
                ctx.shared.borrow().fn_count
            ));
            ctx.increment_fn_count(&node_idx);
            for (i, arg) in args_idx.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }
                pretty_rec(*arg, &mut ctx.with_indent(0).with_nl(""), result);
            }
            result.push_str(") -> ");
            match ret_idx {
                Some(ret_idx) => pretty_rec(
                    *ret_idx,
                    &mut ctx.with_indent(0).with_nl("").show_var_names(false),
                    result,
                ),
                None => result.push_str("()"),
            }
            result.push_str(" {\n");
            for op_idx in body_idx {
                pretty_rec(*op_idx, &mut ctx.add_indent(1).with_in_block(true), result);
            }
            result.push_str(&format!(
                "{}return x{};\n",
                ctx.add_indent(1).indent_str(),
                ctx.shared.borrow().var_count
            ));
            result.push_str(&format!("{}}}", ctx.indent_str()));
            if ctx.shared.borrow().fn_count != ctx.shared.borrow().roots.len() {
                result.push_str("\n\n");
            }
        }
        Operation::Value(spanned_val) => {
            let val = &spanned_val.value;
            match val {
                MirValue::Variable(ty, pos, _func) => {
                    if ctx.in_block {
                        result.push_str(&format!("x{}", pos));
                    } else {
                        if ctx.show_var_names {
                            result.push_str(&format!("{}x{}: ", ctx.indent_str(), pos));
                        };
                        result.push_str(&format!("{:?}{}", ty, ctx.nl));
                    }
                }
                MirValue::Constant(constant) => {
                    result.push_str(&format!("{}{:?}{}", ctx.indent_str(), constant, ctx.nl));
                }
                val => result.push_str(&format!("{}{:?}{}", ctx.indent_str(), val, ctx.nl)),
            };
        }
        Operation::Add(lhs, rhs) => {
            pretty_ssa_2ary((lhs, rhs), ctx, "+", result);
        }
        Operation::Sub(lhs, rhs) => {
            pretty_ssa_2ary((lhs, rhs), ctx, "-", result);
        }
        Operation::Mul(lhs, rhs) => {
            pretty_ssa_2ary((lhs, rhs), ctx, "*", result);
        }
        Operation::Call(func, args) => {
            pretty_ssa_prefix(ctx, result);
            result.push_str(&format!(
                "f{}(",
                ctx.shared.borrow().fns.get(&func.0).unwrap()
            ));
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }
                match ctx.graph.node(arg).op() {
                    Operation::Value(SpannedMirValue {
                        value: MirValue::Variable(_, pos, _),
                        ..
                    }) => result.push_str(&format!("x{}", pos)),
                    _ => pretty_rec(*arg, &mut ctx.with_indent(0).with_nl(""), result),
                }
            }
            pretty_ssa_suffix(ctx, result);
        }
        op => result.push_str(&format!("{}{:?}\n", ctx.indent_str(), op)),
    }
}

fn pretty_ssa_prefix(ctx: &mut PrettyCtx, result: &mut String) {
    result.push_str(&ctx.indent_str());
    ctx.increment_var_count();
    result.push_str(&format!("let x{} = ", ctx.shared.borrow().var_count));
}

fn pretty_ssa_suffix(ctx: &mut PrettyCtx, result: &mut String) {
    result.push_str(&format!(";\n{}", if ctx.in_block { "" } else { ctx.nl }));
}

fn pretty_ssa_2ary(
    (lhs, rhs): (&NodeIndex, &NodeIndex),
    ctx: &mut PrettyCtx,
    op_str: &str,
    result: &mut String,
) {
    pretty_ssa_prefix(ctx, result);
    pretty_rec(*lhs, &mut ctx.add_indent(1).with_nl(""), result);
    result.push_str(&format!(" {} ", op_str));
    pretty_rec(*rhs, &mut ctx.add_indent(1).with_nl(""), result);
    pretty_ssa_suffix(ctx, result);
}
