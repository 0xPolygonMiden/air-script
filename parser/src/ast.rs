use crate::lexer::Token;
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

/// [TraceCols] contains the trace columns representing columns of the execution trace.
/// TraceCols is made up of TraceColsGrp which represents groups of columns of a specific type.
#[derive(Debug, PartialEq)]
pub struct TraceCols {
    pub cols: Vec<TraceColsGrp>,
}

/// [TraceColsGrp] represents a group of columns of a specfic type.
/// There are two types of groups:
/// - MainTraceCols
/// - AuxiliaryTraceCols
#[derive(Debug, PartialEq)]
pub struct TraceColsGrp {
    pub cols_grp_type: TraceColsGrpType,
    pub cols: Vec<Expr>,
}

/// Describes the type of trace columns group.
#[derive(Debug, Eq, PartialEq)]
pub enum TraceColsGrpType {
    MainTraceCols,
    AuxiliaryTraceCols,
}

/// Stores the boundary constraints to be enforced to the trace column values.
#[derive(Debug, PartialEq)]
pub struct BoundaryConstraints {
    pub boundary_constraints: Vec<Constraint>,
}

/// Describes the type of boundary in the boundary constraint.
#[derive(Debug, Eq, PartialEq)]
pub enum Boundary {
    First,
    Last,
}

/// Stores the transition constraints to be enforced to the trace column values.
#[derive(Debug, PartialEq)]
pub struct TransitionConstraints {
    pub transition_constraints: Vec<Constraint>,
}

/// Stores the expression corresponding to the transition or boundary constraint.
#[derive(Debug, PartialEq)]
pub struct Constraint {
    pub expr: Expr,
}

/// Arithmetic expressions representing transition or boundary constraint.
#[derive(Debug, PartialEq)]
pub enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Equal(Box<Expr>, Box<Expr>),
    Boundary(Identifier, Boundary),
    Variable(Identifier),
    Int(Token),
}
