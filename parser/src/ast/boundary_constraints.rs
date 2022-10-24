use std::fmt::Display;

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

impl Display for Boundary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Boundary::First => write!(f, "first boundary"),
            Boundary::Last => write!(f, "last boundary"),
        }
    }
}

/// Arithmetic expressions for evaluation of boundary constraints.
#[derive(Debug, PartialEq, Clone)]
pub enum BoundaryExpr {
    Constant(u64),
    /// Reference to a public input element, identified by the name of a public input array and the
    /// index of the cell.
    PublicInput(Identifier, usize),
    Add(Box<BoundaryExpr>, Box<BoundaryExpr>),
    Subtract(Box<BoundaryExpr>, Box<BoundaryExpr>),
}
