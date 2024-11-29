use crate::ir::{BackLink, IsChild, IsParent, Link, MiddleNode, Node, NodeType};
use std::fmt::Debug;
use std::ops::DerefMut;

#[derive(Clone, Eq, PartialEq, Default)]
pub struct Scope {
    node: Node,
}

impl Scope {
    pub fn new(parent: BackLink<NodeType>, children: Link<Vec<Link<NodeType>>>) -> Self {
        Self {
            node: Node::new(parent, children),
        }
    }
}

impl Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.node)
    }
}

impl IsParent for Scope {
    fn add_child(&mut self, mut child: Link<NodeType>) -> Link<NodeType> {
        // Deduplicate children
        if !self.node.get_children().borrow().contains(&child) {
            self.node.get_children().borrow_mut().push(child.clone());
            child.swap_parent(self.clone().into());
        }
        self.clone().into()
    }
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        self.node.get_children()
    }
}

impl IsChild for Scope {
    fn get_parent(&self) -> BackLink<NodeType> {
        self.node.get_parent()
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        self.node.set_parent(parent);
    }
}

impl From<Scope> for Link<NodeType> {
    fn from(scope: Scope) -> Link<NodeType> {
        Link::new(NodeType::MiddleNode(MiddleNode::Scope(scope)))
    }
}

impl From<Node> for Scope {
    fn from(node: Node) -> Scope {
        let scope = Scope { node };
        let node_children = scope.node.get_children();
        let mut children = Vec::new();

        for child in node_children.borrow().iter() {
            if !node_children.borrow().contains(child) {
                children.push(child.clone());
            }
        }
        *scope.node.get_children().borrow_mut().deref_mut() = children;
        scope
    }
}
