use super::{
    Identifier, ListComprehension, MatrixAccess, TraceAccess, TraceBindingAccess, VectorAccess,
};
use crate::VariableType;

/// Arithmetic expressions for evaluation of constraints.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression {
    Const(u64),
    /// Represents any named constant or variable.
    Elem(Identifier),
    /// Represents an element inside a constant or variable vector. [VectorAccess] contains the
    /// name of the vector and the index of the element to access.
    VectorAccess(VectorAccess),
    /// Represents an element inside a constant or variable matrix. [MatrixAccess] contains the
    /// name of the matrix and indices of the element to access.
    MatrixAccess(MatrixAccess),
    TraceAccess(TraceAccess),
    TraceBindingAccess(TraceBindingAccess),
    /// Represents a random value provided by the verifier. The first inner value is the name of
    /// the random values array and the second is the index of this random value in that array
    Rand(Identifier, usize),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Exp(Box<Expression>, Box<Expression>),
    ListFolding(ListFoldingType),
    FunctionCall(Identifier, Vec<VariableType>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListFoldingType {
    Sum(ListFoldingValueType),
    Prod(ListFoldingValueType),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListFoldingValueType {
    Identifier(Identifier),
    Vector(Vec<Expression>),
    ListComprehension(ListComprehension),
}
