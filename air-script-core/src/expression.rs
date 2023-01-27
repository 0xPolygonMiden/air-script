use super::{Identifier, IndexedTraceAccess, MatrixAccess, NamedTraceAccess, VectorAccess};

/// Arithmetic expressions for evaluation of constraints.
#[derive(Debug, Eq, PartialEq, Clone)]
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
    IndexedTraceAccess(IndexedTraceAccess),
    NamedTraceAccess(NamedTraceAccess),
    /// Represents a random value provided by the verifier. The inner value is the index of this
    /// random value in the array of all random values.
    Rand(usize),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Exp(Box<Expression>, Box<Expression>),
}
