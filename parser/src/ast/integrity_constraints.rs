use super::{Expression, Variable};

// INTEGRITY STATEMENTS
// ================================================================================================

#[derive(Debug, Eq, PartialEq)]
pub enum IntegrityStmt {
    Constraint(IntegrityConstraint),
    Variable(Variable),
}

/// Stores the expression corresponding to the integrity constraint.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct IntegrityConstraint {
    lhs: Expression,
    rhs: Expression,
}

impl IntegrityConstraint {
    pub fn new(lhs: Expression, rhs: Expression) -> Self {
        Self { lhs, rhs }
    }

    /// Clones the left and right internal expressions and creates a single new expression that
    /// represents the integrity constraint when it is equal to zero.
    pub fn expr(&self) -> Expression {
        Expression::Sub(Box::new(self.lhs.clone()), Box::new(self.rhs.clone()))
    }
}
