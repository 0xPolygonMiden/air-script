use block::Block;

use crate::graph::NodeIndex;

use super::*;

/// [Operation] defines the various node types represented
/// in the [MIR].
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operation {
    Block(Block),
    /// Evaluates to a [Value]
    ///
    /// This is always a leaf node in the graph.
    Value(Value),
    /// Evaluates by addition over two operands (given as nodes in the graph)
    Add(NodeIndex, NodeIndex),
    /// Evaluates by subtraction over two operands (given as nodes in the graph)
    Sub(NodeIndex, NodeIndex),
    /// Evaluates by multiplication over two operands (given as nodes in the graph)
    Mul(NodeIndex, NodeIndex),
    /// Enforces a constraint (a given value equals Zero)
    Enf(NodeIndex),
    /// Call a block
    Call(NodeIndex),
}
impl Operation {
    /// Corresponds to the binding power of this [Operation]
    ///
    /// Operations with a higher binding power are applied before
    /// operations with a lower binding power. Operations with equivalent
    /// precedence are evaluated left-to-right.
    pub fn precedence(&self) -> usize {
        match self {
            Self::Add(_, _) => 1,
            Self::Sub(_, _) => 2,
            Self::Mul(_, _) => 3,
            Self::Call(_) => 4,
            Self::Enf(_) => 5,
            Self::Value(_) => 5,
            Self::Block(_) => 5,
        }
    }
}
