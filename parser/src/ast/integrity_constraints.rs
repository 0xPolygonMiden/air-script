use super::{ComprehensionContext, EvaluatorFunctionCall, Expression, VariableBinding};

// INTEGRITY STATEMENTS
// ================================================================================================

#[derive(Debug, Eq, PartialEq)]
pub enum IntegrityStmt {
    Constraint(IntegrityConstraint),
    VariableBinding(VariableBinding),
}

#[derive(Debug, Eq, PartialEq)]
pub struct IntegrityConstraint {
    constraint_expr: ConstraintExpr,
    comprehension_context: Option<ComprehensionContext>,
    selectors: Option<Expression>,
}

impl IntegrityConstraint {
    pub fn new(
        constraint_expr: ConstraintExpr,
        comprehension_context: Option<ComprehensionContext>,
        selectors: Option<Expression>,
    ) -> Self {
        Self {
            constraint_expr,
            comprehension_context,
            selectors,
        }
    }

    pub fn constraint_expr(&self) -> &ConstraintExpr {
        &self.constraint_expr
    }

    pub fn comprehension_context(&self) -> Option<&ComprehensionContext> {
        self.comprehension_context.as_ref()
    }

    pub fn selectors(&self) -> Option<&Expression> {
        self.selectors.as_ref()
    }

    pub fn into_parts(
        self,
    ) -> (
        ConstraintExpr,
        Option<ComprehensionContext>,
        Option<Expression>,
    ) {
        (
            self.constraint_expr,
            self.comprehension_context,
            self.selectors,
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ConstraintExpr {
    Inline(InlineConstraintExpr),
    Evaluator(EvaluatorFunctionCall),
}

/// Stores the expression corresponding to the integrity constraint.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct InlineConstraintExpr {
    lhs: Expression,
    rhs: Expression,
}

impl InlineConstraintExpr {
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
