use crate::ir::{Leaf, LeafNode, Link, NodeType};
use std::fmt::Debug;
#[derive(Clone, Eq, PartialEq, Default)]
pub struct Felt {
    value: i32,
}

impl Felt {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

impl Debug for Felt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Felt({})", self.value)
    }
}

impl From<Leaf<Felt>> for Link<NodeType> {
    fn from(felt: Leaf<Felt>) -> Link<NodeType> {
        Link::new(NodeType::LeafNode(LeafNode::Value(felt)))
    }
}

impl From<i32> for Link<NodeType> {
    fn from(value: i32) -> Link<NodeType> {
        Leaf::new(Felt::new(value)).into()
    }
}
