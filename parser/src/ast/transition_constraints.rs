use super::Expr;

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
    lhs: Expr,
    rhs: Expr,
}

impl TransitionConstraint {
    pub fn new(lhs: Expr, rhs: Expr) -> Self {
        Self { lhs, rhs }
    }

    /// Clones the left and right internal expressions and creates a single new expression that
    /// represents the transition constraint when it is equal to zero.
    pub fn expr(&self) -> Expr {
        Expr::Subtract(Box::new(self.lhs.clone()), Box::new(self.rhs.clone()))
    }
}
