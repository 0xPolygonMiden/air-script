use crate::graph::{Add, BackLink, LeafNode, Link, MiddleNode, NodeType, RootNode};
use std::borrow::BorrowMut;
use std::cell::RefMut;
use std::fmt::Debug;
use std::ops::DerefMut;
use std::rc::Rc;

pub trait Parent: Clone + Into<Link<NodeType>> + Debug {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>>;
    fn add_child(&mut self, mut child: Link<NodeType>) -> Link<NodeType> {
        self.get_children().borrow_mut().push(child.clone());
        child.swap_parent(self.clone().into());
        self.clone().into()
    }
    fn remove_child(&mut self, child: Link<NodeType>) -> Link<NodeType> {
        self.get_children().borrow_mut().retain(|c| c != &child);
        self.clone().into()
    }
    fn first(&self) -> Link<NodeType>
    where
        Self: Debug,
    {
        let children = self.get_children();
        assert!(
            !&children.borrow().is_empty(),
            "first() called on node without children: {:?}",
            self
        );
        let x = children
            .borrow()
            .first()
            .expect("first() called on empty node")
            .clone();
        x
    }
    fn last(&self) -> Link<NodeType>
    where
        Self: Debug,
    {
        let children = self.get_children();
        assert!(
            !&children.borrow().is_empty(),
            "last() called on node without children: {:?}",
            self
        );
        let x = children.borrow().last().unwrap().clone();
        x
    }
    fn new_i32(&mut self, data: i32) -> Link<NodeType> {
        let node: Link<NodeType> = Leaf::new(data).into();
        self.add_child(node.clone());
        node
    }
    fn new_add(&mut self) -> Link<NodeType> {
        let node: Link<NodeType> = Add::new(BackLink::none(), Link::new(Vec::new())).into();
        self.add_child(node.clone());
        node
    }
    fn new_body(&mut self) -> Link<NodeType> {
        let node: Link<NodeType> = Node::new(BackLink::none(), Link::new(Vec::new())).into();
        self.add_child(node.clone());
        node
    }
}

pub trait Child: Clone + Into<Link<NodeType>> + Debug {
    fn get_parent(&self) -> BackLink<NodeType>;
    fn set_parent(&mut self, parent: Link<NodeType>);
    fn swap_parent(&mut self, new_parent: Link<NodeType>) {
        // Grab the old parent before we change it
        let old_parent = self.get_parent().to_link();
        // Remove self from the old parent's children
        if let Some(mut parent) = old_parent {
            if parent != new_parent {
                parent.remove_child(self.clone().into());
            }
        }
        // Change the parent
        self.set_parent(new_parent.into());
        dbg!(&self.get_parent());
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Node {
    parent: BackLink<NodeType>,
    children: Link<Vec<Link<NodeType>>>,
}

impl Node {
    pub fn new(parent: BackLink<NodeType>, children: Link<Vec<Link<NodeType>>>) -> Self {
        Self { parent, children }
    }
}

impl Parent for Node {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        self.children.clone()
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.children)
    }
}

impl Child for Node {
    fn get_parent(&self) -> BackLink<NodeType> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        self.parent = parent.into();
    }
}

impl From<Node> for Link<NodeType> {
    fn from(value: Node) -> Self {
        Link::new(NodeType::MiddleNode(MiddleNode::Node(value)))
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Leaf<T> {
    parent: BackLink<NodeType>,
    data: T,
}

impl<T> Leaf<T> {
    pub fn new(data: T) -> Self {
        Self {
            parent: BackLink::none(),
            data,
        }
    }
}

impl<T: Debug> Debug for Leaf<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.data)
    }
}

impl Child for Leaf<i32> {
    fn get_parent(&self) -> BackLink<NodeType> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        self.parent = parent.into();
    }
}

impl From<Leaf<i32>> for Link<NodeType> {
    fn from(value: Leaf<i32>) -> Self {
        Link::new(NodeType::LeafNode(LeafNode::I32(value)))
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Graph {
    nodes: Link<Vec<Link<NodeType>>>,
}

impl Graph {
    pub fn create() -> Link<NodeType> {
        Link::new(NodeType::RootNode(RootNode::Graph(Graph {
            nodes: Link::new(Vec::new()),
        })))
    }
}

impl Parent for Graph {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        self.nodes.clone()
    }
}

impl Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Graph").field("nodes", &self.nodes).finish()
    }
}

impl From<Graph> for Link<NodeType> {
    fn from(value: Graph) -> Self {
        Link::new(NodeType::RootNode(RootNode::Graph(value)))
    }
}
