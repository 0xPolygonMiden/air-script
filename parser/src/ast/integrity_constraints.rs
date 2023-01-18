use super::{Identifier, IndexedTraceAccess, MatrixAccess, NamedTraceAccess, VectorAccess};

// INTEGRITY CONSTRAINTS
// ================================================================================================

#[derive(Debug, Eq, PartialEq)]
pub enum IntegrityStmt {
    Constraint(IntegrityConstraint),
    Variable(IntegrityVariable),
}

/// Stores the expression corresponding to the integrity constraint.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct IntegrityConstraint {
    lhs: IntegrityExpr,
    rhs: IntegrityExpr,
}

impl IntegrityConstraint {
    pub fn new(lhs: IntegrityExpr, rhs: IntegrityExpr) -> Self {
        Self { lhs, rhs }
    }

    /// Clones the left and right internal expressions and creates a single new expression that
    /// represents the integrity constraint when it is equal to zero.
    pub fn expr(&self) -> IntegrityExpr {
        IntegrityExpr::Sub(Box::new(self.lhs.clone()), Box::new(self.rhs.clone()))
    }
}

/// Arithmetic expressions for evaluation of integrity constraints.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum IntegrityExpr {
    Const(u64),
    /// Represents any named constant or variable.
    Elem(Identifier),
    /// Represents an element inside a constant or variable vector. [VectorAccess] contains the
    /// name of the vector and the index of the element to access.
    VectorAccess(VectorAccess),
    /// Represents an element inside a constant or variable matrix. [MatrixAccess] contains the
    /// name of the matrix and indices of the element to access.
    MatrixAccess(MatrixAccess),
    /// Represents accessing a column by its name in the next row of the execution trace at a particular row offset.
    /// [NamedTraceAccess] contains the name of [TraceCols] and index within it to be accessed.
    Next(NamedTraceAccess),
    /// Represents accessing a column in the execution trace at a particular row offset,
    /// e.g. $main[2]', which accesses the second column of the main trace (segment 0) at the next
    /// row (offset 1).
    /// [IndexedTraceAccess] contains the segment index, column index within the segment, and row
    /// offset of the element to be accessed.
    IndexedTraceAccess(IndexedTraceAccess),
    /// Represents a random value provided by the verifier. The inner value is the index of this
    /// random value in the array of all random values.
    Rand(usize),
    Add(Box<IntegrityExpr>, Box<IntegrityExpr>),
    Sub(Box<IntegrityExpr>, Box<IntegrityExpr>),
    Mul(Box<IntegrityExpr>, Box<IntegrityExpr>),
    Exp(Box<IntegrityExpr>, u64),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct IntegrityVariable {
    name: Identifier,
    value: IntegrityVariableType,
}

impl IntegrityVariable {
    pub fn new(name: Identifier, value: IntegrityVariableType) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> &str {
        self.name.name()
    }

    pub fn value(&self) -> &IntegrityVariableType {
        &self.value
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum IntegrityVariableType {
    Scalar(IntegrityExpr),
    Vector(Vec<IntegrityExpr>),
    Matrix(Vec<Vec<IntegrityExpr>>),
}
