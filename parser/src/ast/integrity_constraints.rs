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

    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }
}
