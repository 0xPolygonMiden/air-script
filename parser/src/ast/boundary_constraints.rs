use super::Identifier;

// BOUNDARY CONSTRAINTS
// ================================================================================================

/// Stores the boundary constraints to be enforced on the trace column values.
#[derive(Debug, PartialEq)]
pub struct BoundaryConstraints {
    pub boundary_constraints: Vec<BoundaryConstraint>,
}

/// Stores the expression corresponding to the boundary constraint.
#[derive(Debug, PartialEq)]
pub struct BoundaryConstraint {
    column: Identifier,
    boundary: Boundary,
    value: BoundaryExpr,
}

impl BoundaryConstraint {
    pub fn new(column: Identifier, boundary: Boundary, value: BoundaryExpr) -> Self {
        Self {
            column,
            boundary,
            value,
        }
    }

    pub fn column(&self) -> &str {
        &self.column.0
    }

    pub fn boundary(&self) -> Boundary {
        self.boundary
    }

    /// Returns a clone of the constraint's value expression.
    pub fn value(&self) -> BoundaryExpr {
        self.value.clone()
    }
}

/// Describes the type of boundary in the boundary constraint.
#[derive(Debug, Eq, Copy, Clone, PartialEq)]
pub enum Boundary {
    First,
    Last,
}

/// Arithmetic expressions for evaluation of boundary constraints.
#[derive(Debug, PartialEq, Clone)]
pub enum BoundaryExpr {
    Const(u64),
    Add(Box<BoundaryExpr>, Box<BoundaryExpr>),
    Sub(Box<BoundaryExpr>, Box<BoundaryExpr>),
    Mul(Box<BoundaryExpr>, Box<BoundaryExpr>),
    Exp(Box<BoundaryExpr>, u64),
}
