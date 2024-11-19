use std::cell::{Ref, RefCell, RefMut};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

trait Parent {
    fn get_children(&self) -> Link<Vec<Link<NodeTypes>>>;
    fn add_child(&mut self, child: Link<NodeTypes>) -> Link<NodeTypes> {
        self.get_children().borrow_mut().push(child.clone());
        child
    }
    fn remove_child(&mut self, child: Link<NodeTypes>) -> Link<NodeTypes> {
        self.get_children().borrow_mut().retain(|c| c != &child);
        child
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
        let node = Link::new(NodeTypes::Add(Node::new(
            BackLink::none(),
            Link::new(Vec::new()),
        )));
        self.add_child(node.clone());
        node
    }
    fn new_node(&mut self) -> Link<NodeTypes> {
        let node = Link::new(NodeTypes::Node(Node::new(
            BackLink::none(),
            Link::new(Vec::new()),
        )));
        self.add_child(node.clone());
        node
    }
}

trait Child {
    fn get_parent(&self) -> BackLink<NodeTypes>;
    fn set_parent(&self, parent: Link<NodeTypes>);
}

struct Link<T>
where
    T: Sized,
{
    link: Rc<RefCell<T>>,
}
impl<T> Link<T> {
    fn new(data: T) -> Self {
        Self {
            link: Rc::new(RefCell::new(data)),
        }
    }
}
impl<T: Debug> Debug for Link<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.link.borrow())
    }
}
impl<T> Clone for Link<T> {
    fn clone(&self) -> Self {
        Self {
            link: self.link.clone(),
        }
    }
}
impl<T> PartialEq for Link<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.link == other.link
    }
}
impl<T> Eq for Link<T> where T: Eq {}

struct BackLink<T> {
    link: Option<Weak<RefCell<T>>>,
}
impl<T> BackLink<T> {
    fn none() -> Self {
        Self { link: None }
    }
    fn to_link(&self) -> Option<Link<T>> {
        match &self.link {
            None => None,
            Some(link) => Some(Link {
                link: link.upgrade().unwrap(),
            }),
        }
    }
}
impl<T> Debug for BackLink<T> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
impl<T> Clone for BackLink<T> {
    fn clone(&self) -> Self {
        Self {
            link: self.link.clone(),
        }
    }
}
impl From<Link<NodeTypes>> for BackLink<NodeTypes> {
    fn from(parent: Link<NodeTypes>) -> Self {
        Self {
            link: Some(Rc::downgrade(&parent.link)),
        }
    }
}
impl<T> PartialEq for BackLink<T> {
    /// Always returns true because the field should be ignored in comparisons.
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<T> Eq for BackLink<T> {}

#[derive(Clone, Eq, PartialEq)]
struct Node {
    parent: BackLink<NodeTypes>,
    children: Link<Vec<Link<NodeTypes>>>,
}
impl Node {
    fn new(parent: BackLink<NodeTypes>, children: Link<Vec<Link<NodeTypes>>>) -> Self {
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

#[derive(Clone, Eq, PartialEq)]
struct Leaf<T> {
    parent: BackLink<NodeTypes>,
    data: T,
}
impl<T> Leaf<T> {
    fn new(data: T) -> Self {
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

#[derive(Clone, Eq, PartialEq)]
struct Graph {
    nodes: Link<Vec<Link<NodeTypes>>>,
}
impl Graph {
    fn new() -> Link<NodeTypes> {
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

#[derive(Clone, Eq, PartialEq)]
enum NodeTypes {
    Graph(Graph),
    Node(Node),
    I32(Leaf<i32>),
    Add(Node),
}
impl Debug for NodeTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeTypes::Graph(graph) => write!(f, "{:?}", graph),
            NodeTypes::Node(node) => write!(f, "{:?}", node),
            NodeTypes::I32(leaf) => write!(f, "{:?}", leaf),
            NodeTypes::Add(add) => write!(f, "add({:?})", add),
        }
    }
}

impl Link<NodeTypes> {
    fn borrow(&self) -> std::cell::Ref<NodeTypes> {
        self.link.borrow()
    }
    fn borrow_mut(&self) -> std::cell::RefMut<NodeTypes> {
        self.link.borrow_mut()
    }
    fn to_back(&self) -> BackLink<NodeTypes> {
        BackLink::from(self.clone())
    }
}
impl Link<Vec<Link<NodeTypes>>> {
    fn borrow(&self) -> std::cell::Ref<Vec<Link<NodeTypes>>> {
        self.link.borrow()
    }
    fn borrow_mut(&self) -> std::cell::RefMut<Vec<Link<NodeTypes>>> {
        self.link.borrow_mut()
    }
}
impl Parent for Link<NodeTypes> {
    /// Add a child to the node.and set the parent of the child to the node.
    /// Returns the node.
    fn add_child(&mut self, child: Link<NodeTypes>) -> Link<NodeTypes> {
        println!("Link<NodeTypes>.add_child({:?})", child);
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
        child.set_parent(self.clone());
        self.clone()
    }
    fn get_children(&self) -> Link<Vec<Link<NodeTypes>>> {
        let children = match self.borrow().deref() {
            NodeTypes::Graph(graph) => graph.get_children(),
            NodeTypes::Node(node) => node.get_children(),
            NodeTypes::Add(add) => add.get_children(),
            NodeTypes::I32(_) => panic!("get_children called on leaf"),
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
        }
    }
    fn first(&self) -> Link<NodeTypes> {
        match self.borrow().deref() {
            NodeTypes::Graph(graph) => graph.first(),
            NodeTypes::Node(node) => node.first(),
            NodeTypes::I32(_) => panic!("first() called on leaf"),
            NodeTypes::Add(add) => add.first(),
        }
    }
    fn last(&self) -> Link<NodeTypes> {
        match self.borrow().deref() {
            NodeTypes::Graph(graph) => graph.last(),
            NodeTypes::Node(node) => node.last(),
            NodeTypes::I32(_) => panic!("last() called on leaf"),
            NodeTypes::Add(add) => add.last(),
        }
    }
    fn new_i32(&mut self, data: i32) -> Link<NodeTypes> {
        let leaf = match self.borrow_mut().deref_mut() {
            NodeTypes::Graph(graph) => graph.new_i32(data),
            NodeTypes::Node(node) => node.new_i32(data),
            NodeTypes::I32(_) => panic!("new_i32() called on leaf"),
            NodeTypes::Add(add) => add.new_i32(data),
        };
        leaf.set_parent(self.clone());
        leaf
    }
    fn new_add(&mut self) -> Link<NodeTypes> {
        let add = match self.borrow_mut().deref_mut() {
            NodeTypes::Graph(graph) => graph.new_add(),
            NodeTypes::Node(node) => node.new_add(),
            NodeTypes::I32(_) => panic!("new_add() called on leaf"),
            NodeTypes::Add(add) => add.new_add(),
        };
        add.set_parent(self.clone());
        add
    }
    fn new_node(&mut self) -> Link<NodeTypes> {
        let node = match self.borrow_mut().deref_mut() {
            NodeTypes::Graph(graph) => graph.new_node(),
            NodeTypes::Node(node) => node.new_node(),
            NodeTypes::I32(_) => panic!("new_node() called on leaf"),
            NodeTypes::Add(add) => add.new_node(),
        };
        node.set_parent(self.clone());
        node
    }
}
impl Child for Link<NodeTypes> {
    fn get_parent(&self) -> BackLink<NodeTypes> {
        match self.borrow().deref() {
            NodeTypes::Graph(_) => panic!("get_parent called on graph"),
            NodeTypes::Node(node) => node.parent.clone(),
            NodeTypes::I32(leaf) => leaf.parent.clone(),
            NodeTypes::Add(add) => add.parent.clone(),
        }
    }
    fn set_parent(&self, parent: Link<NodeTypes>) {
        println!("Link<NodeTypes>.set_parent({:?})", parent);
        if let NodeTypes::Graph(_) = self.borrow().deref() {
            panic!("set_parent called on graph");
        }
        let old_parent = self.get_parent().to_link();
        dbg!(&old_parent);
        if let Some(mut old_parent) = self.get_parent().to_link() {
            dbg!(&old_parent);
            if old_parent != parent {
                old_parent.remove_child(self.clone());
            }
        }
        match self.borrow_mut().deref_mut() {
            NodeTypes::Node(ref mut node) => node.parent = parent.to_back(),
            NodeTypes::I32(ref mut leaf) => leaf.parent = parent.to_back(),
            NodeTypes::Add(ref mut add) => add.parent = parent.to_back(),
            _ => panic!("set_parent called on non-node"),
        }
    }
}

fn main() {
    let mut graph = Graph::new();
    let mut body = graph.new_node();
    body.new_add()
        .add_child(graph.new_i32(1))
        .add_child(graph.new_i32(42));
    dbg!(body);
    println!();
    dbg!(&graph);
    println!();
    graph
        .new_add()
        .add_child(graph.new_i32(2))
        .add_child(graph.new_i32(3));
    dbg!(graph);
}
