use air_script_core::ComprehensionContext;

use super::{EvaluatorFunctionCall, Expression, VariableBinding};

// INTEGRITY STATEMENTS
// ================================================================================================

#[derive(Debug, Eq, PartialEq)]
pub enum IntegrityStmt {
    Constraint(
        ConstraintType,
        Option<Expression>,
        Option<ComprehensionContext>,
    ),
    VariableBinding(VariableBinding),
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

    /// Returns the left-hand side and right-hand side of the integrity constraint as a tuple.
    pub fn into_parts(self) -> (Expression, Expression) {
        (self.lhs, self.rhs)
    }
}
