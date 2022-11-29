use super::{Identifier, MatrixAccess, VectorAccess};

// TRANSITION CONSTRAINTS
// ================================================================================================

/// Stores the transition constraints to be enforced on the trace column values.
#[derive(Debug, PartialEq)]
pub struct TransitionConstraints {
    pub transition_constraints: Vec<TransitionConstraint>,
}

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
    Elem(Identifier),
    VecElem(VectorAccess),
    MatrixElem(MatrixAccess),
    Next(Identifier),
    /// Represents a random value provided by the verifier. The inner value is the index of this
    /// random value in the array of all random values.
    Rand(usize),
    Add(Box<TransitionExpr>, Box<TransitionExpr>),
    Sub(Box<TransitionExpr>, Box<TransitionExpr>),
    Mul(Box<TransitionExpr>, Box<TransitionExpr>),
    Exp(Box<TransitionExpr>, u64),
}
