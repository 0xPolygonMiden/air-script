mod add;
mod felt;
mod function;
mod scope;
use crate::ir2::{BackLink, Graph, IsChild, IsParent, Leaf, Link};
pub use add::Add;
pub use felt::Felt;
pub use function::Function;
pub use scope::Scope;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Eq, PartialEq)]
pub enum RootNode {
    Graph(Graph),
}

impl IsParent for RootNode {
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

impl IsChild for RootNode {
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
    Value(Leaf<Felt>),
}

impl IsParent for LeafNode {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        unreachable!("LeafNode has no children: {:?}", self)
    }
}

impl IsChild for LeafNode {
    fn get_parent(&self) -> BackLink<NodeType> {
        match self {
            LeafNode::Value(leaf) => leaf.get_parent(),
        }
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        match self {
            LeafNode::Value(leaf) => leaf.set_parent(parent),
        }
    }
}

impl From<LeafNode> for Link<NodeType> {
    fn from(leaf_node: LeafNode) -> Link<NodeType> {
        match leaf_node {
            LeafNode::Value(leaf) => leaf.into(),
        }
    }
}

impl Debug for LeafNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeafNode::Value(leaf) => write!(f, "{:?}", leaf),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum MiddleNode {
    Function(Function),
    Add(Add),
    Scope(Scope),
}

impl IsParent for MiddleNode {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        match self {
            MiddleNode::Function(function) => function.get_children(),
            MiddleNode::Add(add) => add.get_children(),
            MiddleNode::Scope(scope) => scope.get_children(),
        }
    }
}

impl IsChild for MiddleNode {
    fn get_parent(&self) -> BackLink<NodeType> {
        match self {
            MiddleNode::Function(function) => function.get_parent(),
            MiddleNode::Add(add) => add.get_parent(),
            MiddleNode::Scope(scope) => scope.get_parent(),
        }
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        match self {
            MiddleNode::Function(function) => function.set_parent(parent),
            MiddleNode::Add(add) => add.set_parent(parent),
            MiddleNode::Scope(scope) => scope.set_parent(parent),
        }
    }
}

impl From<MiddleNode> for Link<NodeType> {
    fn from(middle_node: MiddleNode) -> Link<NodeType> {
        match middle_node {
            MiddleNode::Function(function) => function.into(),
            MiddleNode::Add(add) => add.into(),
            MiddleNode::Scope(scope) => scope.into(),
        }
    }
}

impl Debug for MiddleNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MiddleNode::Function(function) => write!(f, "{:?}", function),
            MiddleNode::Add(add) => write!(f, "{:?}", add),
            MiddleNode::Scope(scope) => write!(f, "{:?}", scope),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum NodeType {
    RootNode(RootNode),
    LeafNode(LeafNode),
    MiddleNode(MiddleNode),
}

impl IsParent for Link<NodeType> {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        match self.borrow().deref() {
            NodeType::LeafNode(leaf_node) => leaf_node.get_children(),
            NodeType::RootNode(root_node) => root_node.get_children(),
            NodeType::MiddleNode(parent_and_child) => parent_and_child.get_children(),
        }
    }
}

impl IsChild for Link<NodeType> {
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
