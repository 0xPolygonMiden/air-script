use crate::graph::{Add, BackLink, Function, Link, NodeTypes};
use std::{
    cell::{RefCell, RefMut},
    fmt::Debug,
    ops::Deref,
    rc::Rc,
};

pub trait Parent: Clone + Into<Link<NodeTypes>> + Debug {
    fn get_children(&self) -> Link<Vec<Link<NodeTypes>>>;
    fn add_child(&mut self, child: Link<NodeTypes>) -> Link<NodeTypes> {
        // This seems to trigger twice when adding a child to a subnode
        println!(
            "{:?}.add_child({:?}, {:?})",
            std::any::type_name::<Self>(),
            self,
            child
        );
        self.get_children().borrow_mut().push(child.clone());
        self.clone().into()
    }
    fn remove_child(&mut self, child: Link<NodeTypes>) -> Link<NodeTypes> {
        self.get_children().borrow_mut().retain(|c| c != &child);
        self.clone().into()
    }
    fn first(&self) -> Link<NodeTypes>
    where
        Self: Debug,
    {
        let children = self.get_children();
        assert!(
            !&children.borrow().is_empty(),
            "first() called on node without children: {:?}",
            self
        );
        let x = children.borrow()[0].clone();
        x
    }
    fn last(&self) -> Link<NodeTypes>
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
    fn new_i32(&mut self, data: i32) -> Link<NodeTypes> {
        let node = Link::new(NodeTypes::I32(Leaf::new(data)));
        self.add_child(node.clone());
        node
    }
    fn new_add(&mut self) -> Link<NodeTypes> {
        let node = Link::new(NodeTypes::Add(Add::new(
            BackLink::none(),
            Link::new(Vec::new()),
        )));
        self.add_child(node.clone());
        node
    }
    fn new_body(&mut self) -> Link<NodeTypes> {
        let node = Link::new(NodeTypes::Node(Node::new(
            BackLink::none(),
            Link::new(Vec::new()),
        )));
        self.add_child(node.clone());
        node
    }
}

pub trait Child: Clone + Into<Link<NodeTypes>> + Debug {
    fn get_parent(&self) -> BackLink<NodeTypes>;
    fn swap_parent(&mut self, new_parent: Link<NodeTypes>) {
        println!("swap_parent({:?}, {:?})", self, new_parent);
        // Grab the old parent before we change it
        let old_parent = self.get_parent().to_link();
        // Remove self from the old parent's children
        if let Some(mut parent) = old_parent {
            if parent != new_parent {
                parent.remove_child(self.clone().into());
            }
        }
        // Change the parent
        self.get_parent().link = Some(Rc::downgrade(&new_parent.link));
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Node {
    parent: BackLink<NodeTypes>,
    children: Link<Vec<Link<NodeTypes>>>,
}
impl Node {
    pub fn new(parent: BackLink<NodeTypes>, children: Link<Vec<Link<NodeTypes>>>) -> Self {
        Self { parent, children }
    }
}
impl Parent for Node {
    fn get_children(&self) -> Link<Vec<Link<NodeTypes>>> {
        self.children.clone()
    }
}
impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.children)
    }
}
impl Child for Node {
    fn get_parent(&self) -> BackLink<NodeTypes> {
        self.parent.clone()
    }
}
impl From<Node> for Link<NodeTypes> {
    fn from(value: Node) -> Self {
        Link::new(NodeTypes::Node(value))
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Leaf<T> {
    parent: BackLink<NodeTypes>,
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
    fn get_parent(&self) -> BackLink<NodeTypes> {
        self.parent.clone()
    }
}
impl From<Leaf<i32>> for Link<NodeTypes> {
    fn from(value: Leaf<i32>) -> Self {
        Link::new(NodeTypes::I32(value))
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Graph {
    nodes: Link<Vec<Link<NodeTypes>>>,
}
impl Graph {
    pub fn create() -> Link<NodeTypes> {
        Link::new(NodeTypes::Graph(Self {
            nodes: Link::new(Vec::new()),
        }))
    }
}
impl Parent for Graph {
    fn get_children(&self) -> Link<Vec<Link<NodeTypes>>> {
        self.nodes.clone()
    }
}
impl Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Graph").field("nodes", &self.nodes).finish()
    }
}

impl From<Graph> for Link<NodeTypes> {
    fn from(value: Graph) -> Self {
        Link::new(NodeTypes::Graph(value))
    }
}
