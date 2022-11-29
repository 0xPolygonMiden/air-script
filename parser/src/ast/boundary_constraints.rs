use super::{Identifier, MatrixAccess, VectorAccess};
use std::fmt::Display;

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
    Const(u64),
    /// Represents any named constant or variable.
    Elem(Identifier),
    /// Represents an element inside a constant or variable vector. The index is the index of
    /// this value inside the vector.
    VecElem(VectorAccess),
    /// Represents an element inside a constant or variable matrix. Indices idx_row and idx_col
    /// are the indices of this value inside the matrix.
    MatrixElem(MatrixAccess),
    /// Represents a random value provided by the verifier. The inner value is the index of this
    /// random value in the array of all random values.
    Rand(usize),
    Add(Box<BoundaryExpr>, Box<BoundaryExpr>),
    Sub(Box<BoundaryExpr>, Box<BoundaryExpr>),
    Mul(Box<BoundaryExpr>, Box<BoundaryExpr>),
    Exp(Box<BoundaryExpr>, u64),
}
