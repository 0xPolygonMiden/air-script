use std::fmt;

// AST
// ================================================================================================

/// [Source] is the root node of the AST representing the AIR constraints file.
#[derive(Debug, PartialEq)]
pub struct Source(pub Vec<SourceSection>);

/// Source is divided into SourceSections.
/// There are 4 types of Source Sections:
/// - AirDef: Name of the air constraints module.
/// - TraceCols: Trace Columns representing columns of the execution trace.
/// - BoundaryConstraints: Boundary Constraints to be enforced on the boundaries of columns defined
///   in the TraceCols section. Currently there are two types of boundaries, First and Last
///   representing the first and last rows of the column.
/// - TransitionConstraints: Transition Constraints to be enforced on the trace columns defined
///   in the TraceCols section.
#[derive(Debug, PartialEq)]
pub enum SourceSection {
    AirDef(Identifier),
    TraceCols(TraceCols),
    BoundaryConstraints(BoundaryConstraints),
    TransitionConstraints(TransitionConstraints),
}

// TRACE
// ================================================================================================

/// [TraceCols] contains the main and auxiliary trace columns of the execution trace.
#[derive(Debug, Eq, PartialEq)]
pub struct TraceCols {
    pub main_cols: Vec<Identifier>,
    pub aux_cols: Vec<Identifier>,
}

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
    value: Expr,
}

impl BoundaryConstraint {
    pub fn new(column: Identifier, boundary: Boundary, value: Expr) -> Self {
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
    pub fn value(&self) -> Expr {
        self.value.clone()
    }
}

/// Describes the type of boundary in the boundary constraint.
#[derive(Debug, Eq, Copy, Clone, PartialEq)]
pub enum Boundary {
    First,
    Last,
}

// TRANSITION CONSTRAINTS
// ================================================================================================

/// Stores the transition constraints to be enforced on the trace column values.
#[derive(Debug, PartialEq)]
pub struct TransitionConstraints {
    pub transition_constraints: Vec<TransitionConstraint>,
}

/// Stores the expression corresponding to the transition constraint.
#[derive(Debug, PartialEq)]
pub struct TransitionConstraint {
    lhs: Expr,
    rhs: Expr,
}

impl TransitionConstraint {
    pub fn new(lhs: Expr, rhs: Expr) -> Self {
        Self { lhs, rhs }
    }

    /// Clones the left and right internal expressions and creates a single new expression that
    /// represents the transition constraint when it is equal to zero.
    pub fn expr(&self) -> Expr {
        Expr::Subtract(Box::new(self.lhs.clone()), Box::new(self.rhs.clone()))
    }
}

/// Arithmetic expressions for constraint evaluation.
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Constant(u64),
    Variable(Identifier),
    Next(Identifier),
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
}

/// [Identifier] is used to represent variable names.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct Identifier(pub String);

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}
