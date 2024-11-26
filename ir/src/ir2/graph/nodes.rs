use crate::graph::{BackLink, Child, Graph, Leaf, Link, Node, Parent};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Eq, PartialEq)]
pub enum RootNode {
    Graph(Graph),
}

impl Parent for RootNode {
    fn add_child(&mut self, child: Link<NodeType>) -> Link<NodeType> {
        match self {
            RootNode::Graph(graph) => graph.add_child(child),
        }
    }
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        match self {
            RootNode::Graph(graph) => graph.get_children(),
        }
    }
}

impl Child for RootNode {
    fn get_parent(&self) -> BackLink<NodeType> {
        unreachable!("RootNode has no parent: {:?}", self)
    }
    fn set_parent(&mut self, _parent: Link<NodeType>) {
        unreachable!("RootNode has no parent: {:?}", self)
    }
}

impl From<RootNode> for Link<NodeType> {
    fn from(root_node: RootNode) -> Link<NodeType> {
        match root_node {
            RootNode::Graph(graph) => Link::new(NodeType::RootNode(RootNode::Graph(graph))),
        }
    }
}

impl Debug for RootNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RootNode::Graph(graph) => write!(f, "{:?}", graph),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum LeafNode {
    I32(Leaf<i32>),
}

impl Parent for LeafNode {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        unreachable!("LeafNode has no children: {:?}", self)
    }
}

impl Child for LeafNode {
    fn get_parent(&self) -> BackLink<NodeType> {
        match self {
            LeafNode::I32(leaf) => leaf.get_parent(),
        }
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        match self {
            LeafNode::I32(leaf) => leaf.set_parent(parent),
        }
    }
}

impl From<LeafNode> for Link<NodeType> {
    fn from(leaf_node: LeafNode) -> Link<NodeType> {
        match leaf_node {
            LeafNode::I32(leaf) => leaf.into(),
        }
    }
}

impl Debug for LeafNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeafNode::I32(leaf) => write!(f, "{:?}", leaf),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum MiddleNode {
    Node(Node),
    Function(Function),
    Add(Add),
}

impl Parent for MiddleNode {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        match self {
            MiddleNode::Node(node) => node.get_children(),
            MiddleNode::Function(function) => function.get_children(),
            MiddleNode::Add(add) => add.get_children(),
        }
    }
}

impl Child for MiddleNode {
    fn get_parent(&self) -> BackLink<NodeType> {
        match self {
            MiddleNode::Node(node) => node.get_parent(),
            MiddleNode::Function(function) => function.get_parent(),
            MiddleNode::Add(add) => add.get_parent(),
        }
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        match self {
            MiddleNode::Node(node) => node.set_parent(parent),
            MiddleNode::Function(function) => function.set_parent(parent),
            MiddleNode::Add(add) => add.set_parent(parent),
        }
    }
}

impl From<MiddleNode> for Link<NodeType> {
    fn from(middle_node: MiddleNode) -> Link<NodeType> {
        match middle_node {
            MiddleNode::Node(node) => node.into(),
            MiddleNode::Function(function) => function.into(),
            MiddleNode::Add(add) => add.into(),
        }
    }
}

impl Debug for MiddleNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MiddleNode::Node(node) => write!(f, "{:?}", node),
            MiddleNode::Function(function) => write!(f, "{:?}", function),
            MiddleNode::Add(add) => write!(f, "{:?}", add),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum NodeType {
    RootNode(RootNode),
    LeafNode(LeafNode),
    MiddleNode(MiddleNode),
}

impl Parent for Link<NodeType> {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        match self.borrow().deref() {
            NodeType::LeafNode(leaf_node) => leaf_node.get_children(),
            NodeType::RootNode(root_node) => root_node.get_children(),
            NodeType::MiddleNode(parent_and_child) => parent_and_child.get_children(),
        }
    }
}

impl Child for Link<NodeType> {
    fn get_parent(&self) -> BackLink<NodeType> {
        match self.borrow().deref() {
            NodeType::LeafNode(leaf_node) => leaf_node.get_parent(),
            NodeType::RootNode(root_node) => root_node.get_parent(),
            NodeType::MiddleNode(parent_and_child) => parent_and_child.get_parent(),
        }
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        match self.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.set_parent(parent),
            NodeType::RootNode(root_node) => root_node.set_parent(parent),
            NodeType::MiddleNode(parent_and_child) => parent_and_child.set_parent(parent),
        }
    }
}

impl Debug for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::LeafNode(leaf_node) => write!(f, "{:?}", leaf_node),
            NodeType::RootNode(root_node) => write!(f, "{:?}", root_node),
            NodeType::MiddleNode(parent_and_child) => write!(f, "{:?}", parent_and_child),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Function {
    node: Node,
}

impl Function {
    pub fn new(args: Link<NodeType>, ret: Link<NodeType>, body: Link<NodeType>) -> Self {
        Self {
            node: Node::new(BackLink::none(), Link::new(vec![args, ret, body])),
        }
    }
    pub fn args(&self) -> Link<NodeType> {
        self.node.get_children().borrow().deref()[0].clone()
    }
    pub fn ret(&self) -> Link<NodeType> {
        self.node.get_children().borrow().deref()[1].clone()
    }
    pub fn body(&self) -> Link<NodeType> {
        self.node.get_children().borrow().deref()[2].clone()
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Function(args: {:?}, ret: {:?}, body: {:?})",
            self.args(),
            self.ret(),
            self.body()
        )
    }
}

impl Parent for Function {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        self.node.get_children()
    }
}

impl Child for Function {
    fn get_parent(&self) -> BackLink<NodeType> {
        self.node.get_parent()
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        self.node.set_parent(parent);
    }
}

impl From<Function> for Link<NodeType> {
    fn from(function: Function) -> Link<NodeType> {
        Link::new(NodeType::MiddleNode(MiddleNode::Function(function)))
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Add {
    node: Node,
}

impl Add {
    pub fn new(parent: BackLink<NodeType>, children: Link<Vec<Link<NodeType>>>) -> Self {
        Self {
            node: Node::new(parent, children),
        }
    }
}

impl Debug for Add {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Add{:?}", &self.node)
    }
}

impl Parent for Add {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        self.node.get_children()
    }
}

impl Child for Add {
    fn get_parent(&self) -> BackLink<NodeType> {
        self.node.get_parent()
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        self.node.set_parent(parent);
    }
}

impl From<Add> for Link<NodeType> {
    fn from(add: Add) -> Link<NodeType> {
        Link::new(NodeType::MiddleNode(MiddleNode::Add(add)))
    }
}
