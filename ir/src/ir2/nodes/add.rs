use crate::ir::{IsNode, Node};

#[derive(Clone, Eq, PartialEq, Default, IsNode)]
pub struct Add {
    #[node(lhs, rhs)]
    node: Node,
}
