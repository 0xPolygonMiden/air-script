use crate::graph::{BackLink, Child, Graph, Leaf, Link, Node, Parent};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Eq, PartialEq)]
pub enum NodeTypes {
    Graph(Graph),
    Node(Node),
    I32(Leaf<i32>),
    Add(Add),
    Function(Function),
}

impl Debug for NodeTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeTypes::Graph(graph) => write!(f, "{:?}", graph),
            NodeTypes::Node(node) => write!(f, "{:?}", node),
            NodeTypes::I32(leaf) => write!(f, "{:?}", leaf),
            NodeTypes::Add(add) => write!(f, "{:?}", add),
            NodeTypes::Function(function) => write!(f, "{:?}", function),
        }
    }
}

impl Parent for Link<NodeTypes> {
    /// Add a child to the node.and set the parent of the child to the node.
    /// Returns the node.
    fn add_child(&mut self, child: Link<NodeTypes>) -> Link<NodeTypes> {
        println!(
            "{:?}.add_child({:?}, {:?})",
            std::any::type_name::<Self>(),
            self,
            child
        );
        let mut not_a_node = false;
        match self.borrow_mut().deref_mut() {
            NodeTypes::Graph(graph) => {
                graph.add_child(child.clone());
            }
            NodeTypes::Node(node) => {
                node.add_child(child.clone());
            }
            NodeTypes::Add(add) => {
                add.add_child(child.clone());
            }
            _ => {
                // a self field is mutably borrowed and dereferenced
                // so self is not available for borrowing and debugging
                not_a_node = true;
            }
        }
        if not_a_node {
            panic!("add_child called on non-node: {:?}", self);
        }
        self.clone()
    }
    fn get_children(&self) -> Link<Vec<Link<NodeTypes>>> {
        let children = match self.borrow().deref() {
            NodeTypes::Graph(graph) => graph.get_children(),
            NodeTypes::Node(node) => node.get_children(),
            NodeTypes::Add(add) => add.get_children(),
            NodeTypes::I32(_) => panic!("get_children called on leaf"),
            NodeTypes::Function(function) => function.get_children(),
        };
        children
    }
    fn remove_child(&mut self, child: Link<NodeTypes>) -> Link<NodeTypes> {
        println!("Link<NodeTypes>.remove_child({:?})", child);
        match self.borrow_mut().deref_mut() {
            NodeTypes::Graph(graph) => graph.remove_child(child),
            NodeTypes::Node(node) => node.remove_child(child),
            NodeTypes::I32(_) => panic!("remove_child called on leaf"),
            NodeTypes::Add(add) => add.remove_child(child),
            NodeTypes::Function(function) => function.body().remove_child(child),
        }
    }
    fn first(&self) -> Link<NodeTypes> {
        match self.borrow().deref() {
            NodeTypes::Graph(graph) => graph.first(),
            NodeTypes::Node(node) => node.first(),
            NodeTypes::I32(_) => panic!("first() called on leaf"),
            NodeTypes::Add(add) => add.first(),
            NodeTypes::Function(function) => function.body().first(),
        }
    }
    fn last(&self) -> Link<NodeTypes> {
        match self.borrow().deref() {
            NodeTypes::Graph(graph) => graph.last(),
            NodeTypes::Node(node) => node.last(),
            NodeTypes::I32(_) => panic!("last() called on leaf"),
            NodeTypes::Add(add) => add.last(),
            NodeTypes::Function(function) => function.body().last(),
        }
    }
    fn new_i32(&mut self, data: i32) -> Link<NodeTypes> {
        match self.borrow_mut().deref_mut() {
            NodeTypes::Graph(graph) => graph.new_i32(data),
            NodeTypes::Node(node) => node.new_i32(data),
            NodeTypes::I32(_) => panic!("new_i32() called on leaf"),
            NodeTypes::Add(add) => add.new_i32(data),
            NodeTypes::Function(function) => function.body().new_i32(data),
        }
    }
    fn new_add(&mut self) -> Link<NodeTypes> {
        match self.borrow_mut().deref_mut() {
            NodeTypes::Graph(graph) => graph.new_add(),
            NodeTypes::Node(node) => node.new_add(),
            NodeTypes::I32(_) => panic!("new_add() called on leaf"),
            NodeTypes::Add(add) => add.new_add(),
            NodeTypes::Function(function) => function.new_add(),
        }
    }
    fn new_body(&mut self) -> Link<NodeTypes> {
        match self.borrow_mut().deref_mut() {
            NodeTypes::Graph(graph) => graph.new_body(),
            NodeTypes::Node(_) => panic!("new_body() called on body"),
            NodeTypes::I32(_) => panic!("new_body() called on leaf"),
            NodeTypes::Add(_) => panic!("new_body() called on add"),
            NodeTypes::Function(_) => panic!("new_body() called on function"),
        }
    }
}

impl Child for Link<NodeTypes> {
    fn get_parent(&self) -> BackLink<NodeTypes> {
        match self.borrow().deref() {
            NodeTypes::Graph(_) => panic!("get_parent called on graph"),
            NodeTypes::Node(node) => node.get_parent(),
            NodeTypes::I32(leaf) => leaf.get_parent(),
            NodeTypes::Add(add) => add.get_parent(),
            NodeTypes::Function(function) => function.get_parent(),
        }
    }
    fn swap_parent(&mut self, parent: Link<NodeTypes>) {
        println!("Link<NodeTypes>.set_parent({:?}, {:?})", self, parent);
        match self.borrow_mut().deref_mut() {
            NodeTypes::Node(ref mut node) => node.swap_parent(parent),
            NodeTypes::I32(ref mut leaf) => leaf.swap_parent(parent),
            NodeTypes::Add(ref mut add) => add.swap_parent(parent),
            NodeTypes::Function(ref mut function) => function.swap_parent(parent),
            _ => panic!("set_parent called on non-node"),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Function {
    node: Node,
}

impl Function {
    pub fn new(args: Link<NodeTypes>, ret: Link<NodeTypes>, body: Link<NodeTypes>) -> Self {
        Self {
            node: Node::new(BackLink::none(), Link::new(vec![args, ret, body])),
        }
    }
    pub fn args(&self) -> Link<NodeTypes> {
        self.node.get_children().borrow().deref()[0].clone()
    }
    pub fn ret(&self) -> Link<NodeTypes> {
        self.node.get_children().borrow().deref()[1].clone()
    }
    pub fn body(&self) -> Link<NodeTypes> {
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
    fn get_children(&self) -> Link<Vec<Link<NodeTypes>>> {
        self.node.get_children()
    }
}

impl Child for Function {
    fn get_parent(&self) -> BackLink<NodeTypes> {
        self.node.get_parent()
    }
    fn swap_parent(&mut self, parent: Link<NodeTypes>) {
        self.node.swap_parent(parent)
    }
}
impl From<Function> for Link<NodeTypes> {
    fn from(function: Function) -> Link<NodeTypes> {
        Link::new(NodeTypes::Function(function))
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Add {
    pub node: Node,
}

impl Add {
    pub fn new(parent: BackLink<NodeTypes>, children: Link<Vec<Link<NodeTypes>>>) -> Self {
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
    fn get_children(&self) -> Link<Vec<Link<NodeTypes>>> {
        self.node.get_children()
    }
}

impl Child for Add {
    fn get_parent(&self) -> BackLink<NodeTypes> {
        self.node.get_parent()
    }
}

impl From<Add> for Link<NodeTypes> {
    fn from(add: Add) -> Link<NodeTypes> {
        Link::new(NodeTypes::Add(add))
    }
}
