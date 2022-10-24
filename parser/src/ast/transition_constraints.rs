use super::Identifier;

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
        TransitionExpr::Subtract(Box::new(self.lhs.clone()), Box::new(self.rhs.clone()))
    }
}

/// Arithmetic expressions for evaluation of transition constraints.
#[derive(Debug, PartialEq, Clone)]
pub enum TransitionExpr {
    Constant(u64),
    Variable(Identifier),
    Next(Identifier),
    Add(Box<TransitionExpr>, Box<TransitionExpr>),
    Subtract(Box<TransitionExpr>, Box<TransitionExpr>),
}