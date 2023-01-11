use super::{Identifier, MatrixAccess, NamedTraceAccess, VectorAccess};
use std::fmt::Display;

// BOUNDARY CONSTRAINTS
// ================================================================================================

#[derive(Debug, Eq, PartialEq)]
pub enum BoundaryStmt {
    Constraint(BoundaryConstraint),
    Variable(BoundaryVariable),
}

/// Stores the expression corresponding to the boundary constraint.
#[derive(Debug, Eq, PartialEq)]
pub struct BoundaryConstraint {
    column: NamedTraceAccess,
    boundary: Boundary,
    value: BoundaryExpr,
}

impl BoundaryConstraint {
    pub fn new(column: NamedTraceAccess, boundary: Boundary, value: BoundaryExpr) -> Self {
        Self {
            column,
            boundary,
            value,
        }
    }

    pub fn column(&self) -> &NamedTraceAccess {
        &self.column
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
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BoundaryExpr {
    Const(u64),
    /// Represents any named constant or variable.
    Elem(Identifier),
    /// Represents an element inside a constant or variable vector. [VectorAccess] contains the
    /// name of the vector and the index of the element to access.
    VectorAccess(VectorAccess),
    /// Represents an element inside a constant or variable matrix. [MatrixAccess] contains the
    /// name of the matrix and indices of the element to access.
    MatrixAccess(MatrixAccess),
    /// Represents a random value provided by the verifier. The inner value is the index of this
    /// random value in the array of all random values.
    Rand(usize),
    Add(Box<BoundaryExpr>, Box<BoundaryExpr>),
    Sub(Box<BoundaryExpr>, Box<BoundaryExpr>),
    Mul(Box<BoundaryExpr>, Box<BoundaryExpr>),
    Exp(Box<BoundaryExpr>, u64),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BoundaryVariable {
    name: Identifier,
    value: BoundaryVariableType,
}

impl BoundaryVariable {
    pub fn new(name: Identifier, value: BoundaryVariableType) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> &str {
        self.name.name()
    }

    pub fn value(&self) -> &BoundaryVariableType {
        &self.value
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BoundaryVariableType {
    Scalar(BoundaryExpr),
    Vector(Vec<BoundaryExpr>),
    Matrix(Vec<Vec<BoundaryExpr>>),
}
