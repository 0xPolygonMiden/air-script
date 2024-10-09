
use value::SpannedMirValue;

use crate::graph::NodeIndex;

use super::*;

/// [Operation] defines the various node types represented
/// in the [MIR].
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operation {
    /// Begin primitive operations

    /// Evaluates to a [TypedValue]
    Value(SpannedMirValue),
    /// Evaluates by addition over two operands (given as nodes in the graph)
    Add(NodeIndex, NodeIndex),
    /// Evaluates by subtraction over two operands (given as nodes in the graph)
    Sub(NodeIndex, NodeIndex),
    /// Evaluates by multiplication over two operands (given as nodes in the graph)
    Mul(NodeIndex, NodeIndex),
    /// Enforces a constraint (a given value equals Zero)
    Enf(NodeIndex),

    /// Begin structured operations
    /// Call (body, arguments)
    Call(NodeIndex, Vec<NodeIndex>),
    /// Fold an Iterator according to a given FoldOperator and a given initial value 
    Fold(NodeIndex, FoldOperator, NodeIndex),
    /// For (iterator, body)
    For(NodeIndex, NodeIndex),
    /// If (condition, then, else)
    If(NodeIndex, NodeIndex, NodeIndex),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FoldOperator {
    Add,
    Mul,
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
            Self::Call(_, _) => 4,
            Self::Value(_) => 5,
            Self::Enf(_) => 6,
            _ => 0

        }
    }

}
