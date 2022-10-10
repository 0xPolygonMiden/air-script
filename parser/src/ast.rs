use std::fmt;

/// [Identifier] is used to represent variable names.
#[derive(Debug, Eq, PartialEq)]
pub struct Identifier {
    pub name: String,
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.name)
    }
}

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
    AirDef(AirDef),
    TraceCols(TraceCols),
    BoundaryConstraints(BoundaryConstraints),
    TransitionConstraints(TransitionConstraints),
}

/// Name of the air constraints module.
#[derive(Debug, Eq, PartialEq)]
pub struct AirDef {
    pub name: Identifier,
}

/// [TraceCols] contains the main and auxiliary trace columns of the execution trace.
#[derive(Debug, Eq, PartialEq)]
pub struct TraceCols {
    pub main_cols: Vec<Identifier>,
    pub aux_cols: Vec<Identifier>,
}

/// Stores the boundary constraints to be enforced on the trace column values.
#[derive(Debug, PartialEq)]
pub struct BoundaryConstraints {
    pub boundary_constraints: Vec<BoundaryConstraint>,
}

/// Stores the expression corresponding to the boundary constraint.
#[derive(Debug, PartialEq)]
pub struct BoundaryConstraint {
    pub column: Identifier,
    pub boundary: Boundary,
    pub value: Expr,
}

/// Describes the type of boundary in the boundary constraint.
#[derive(Debug, Eq, PartialEq)]
pub enum Boundary {
    First,
    Last,
}

/// Stores the transition constraints to be enforced on the trace column values.
#[derive(Debug, PartialEq)]
pub struct TransitionConstraints {
    pub transition_constraints: Vec<TransitionConstraint>,
}

/// Stores the expression corresponding to the transition constraint.
#[derive(Debug, PartialEq)]
pub struct TransitionConstraint {
    pub lhs: Expr,
    pub rhs: Expr,
}

/// Arithmetic expressions for constraint evaluation.
#[derive(Debug, PartialEq)]
pub enum Expr {
    Constant(u64),
    Variable(Identifier),
    Next(Identifier),
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
}
