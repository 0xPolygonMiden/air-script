use super::{EvaluatorFunctionCall, Expression, Identifier, Iterable, Variable};

// INTEGRITY STATEMENTS
// ================================================================================================

#[derive(Debug, Eq, PartialEq)]
pub enum IntegrityStmt {
    Constraint(ConstraintType, Option<Expression>),
    ConstraintComprehension(IntegrityConstraintComprehension),
    Variable(Variable),
}

#[derive(Debug, Eq, PartialEq)]
pub enum ConstraintType {
    Inline(IntegrityConstraint),
    Evaluator(EvaluatorFunctionCall),
}

/// Stores the expression corresponding to the integrity constraint.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct IntegrityConstraint {
    lhs: Expression,
    rhs: Expression,
}

impl IntegrityConstraint {
    /// Creates a new integrity constraint.
    pub fn new(lhs: Expression, rhs: Expression) -> Self {
        Self { lhs, rhs }
    }

    /// Returns the left-hand side of the integrity constraint.
    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    /// Returns the right-hand side of the integrity constraint.
    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct IntegrityConstraintComprehension {
    lhs: Expression,
    rhs: Expression,
    context: Vec<(Identifier, Iterable)>,
}

impl IntegrityConstraintComprehension {
    pub fn new(lhs: Expression, rhs: Expression, context: Vec<(Identifier, Iterable)>) -> Self {
        Self { lhs, rhs, context }
    }

    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }

    pub fn context(&self) -> &[(Identifier, Iterable)] {
        &self.context
    }
}
