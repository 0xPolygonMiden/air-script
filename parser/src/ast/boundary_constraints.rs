use super::{ComprehensionContext, Expression, SymbolAccess, VariableBinding};
use std::fmt::Display;

// BOUNDARY STATEMENTS
// ================================================================================================

#[derive(Debug, Eq, PartialEq)]
pub enum BoundaryStmt {
    Constraint(BoundaryConstraint),
    VariableBinding(VariableBinding),
}

/// Stores the expression corresponding to the boundary constraint.
#[derive(Debug, Eq, PartialEq)]
pub struct BoundaryConstraint {
    access: SymbolAccess,
    boundary: Boundary,
    value: Expression,
    comprehension_context: Option<ComprehensionContext>,
}

impl BoundaryConstraint {
    pub fn new(
        access: SymbolAccess,
        boundary: Boundary,
        value: Expression,
        comprehension_context: Option<ComprehensionContext>,
    ) -> Self {
        Self {
            access,
            boundary,
            value,
            comprehension_context,
        }
    }

    pub fn access(&self) -> &SymbolAccess {
        &self.access
    }

    pub fn boundary(&self) -> Boundary {
        self.boundary
    }

    /// Returns the constraint's value expression.
    pub fn value(&self) -> &Expression {
        &self.value
    }

    pub fn comprehension_context(&self) -> Option<&ComprehensionContext> {
        self.comprehension_context.as_ref()
    }

    pub fn into_parts(
        self,
    ) -> (
        Boundary,
        SymbolAccess,
        Expression,
        Option<ComprehensionContext>,
    ) {
        (
            self.boundary,
            self.access,
            self.value,
            self.comprehension_context,
        )
    }
}

/// Describes the type of boundary in the boundary constraint.
#[derive(Debug, Eq, Copy, Clone, PartialEq)]
pub enum Boundary {
    First,
    Last,
}

impl Display for Boundary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Boundary::First => write!(f, "first boundary"),
            Boundary::Last => write!(f, "last boundary"),
        }
    }
}
