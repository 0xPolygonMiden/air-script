use super::{Identifier, MatrixAccess, VectorAccess};

// TRANSITION CONSTRAINTS
// ================================================================================================

/// Stores the expression corresponding to the transition constraint.
#[derive(Debug, PartialEq, Clone)]
pub struct TransitionConstraint {
    lhs: TransitionExpr,
    rhs: TransitionExpr,
}

impl TransitionConstraint {
    pub fn new(lhs: TransitionExpr, rhs: TransitionExpr) -> Self {
        Self { lhs, rhs }
    }

    /// Clones the left and right internal expressions and creates a single new expression that
    /// represents the transition constraint when it is equal to zero.
    pub fn expr(&self) -> TransitionExpr {
        TransitionExpr::Sub(Box::new(self.lhs.clone()), Box::new(self.rhs.clone()))
    }
}

/// Arithmetic expressions for evaluation of transition constraints.
#[derive(Debug, PartialEq, Clone)]
pub enum TransitionExpr {
    Const(u64),
    /// Represents any named constant or variable.
    Elem(Identifier),
    /// Represents an element inside a constant or variable vector. [VectorAccess] contains the
    /// name of the vector and the index of the element to access.
    VectorAccess(VectorAccess),
    /// Represents an element inside a constant or variable matrix. [MatrixAccess] contains the
    /// name of the matrix and indices of the element to access.
    MatrixAccess(MatrixAccess),
    Next(Identifier),
    /// Represents a random value provided by the verifier. The inner value is the index of this
    /// random value in the array of all random values.
    Rand(usize),
    Add(Box<TransitionExpr>, Box<TransitionExpr>),
    Sub(Box<TransitionExpr>, Box<TransitionExpr>),
    Mul(Box<TransitionExpr>, Box<TransitionExpr>),
    Exp(Box<TransitionExpr>, u64),
}

#[derive(Debug, PartialEq)]
pub struct TransitionVariable {
    name: Identifier,
    value: TransitionVariableType,
}

impl TransitionVariable {
    pub fn new(name: Identifier, value: TransitionVariableType) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, PartialEq)]
pub enum TransitionVariableType {
    Scalar(TransitionExpr),
    Vector(Vec<TransitionExpr>),
    Matrix(Vec<Vec<TransitionExpr>>),
}

#[derive(Debug, PartialEq)]
pub enum TransitionStmt {
    Constraint(TransitionConstraint),
    Variable(TransitionVariable),
}
