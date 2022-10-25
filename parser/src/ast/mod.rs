use std::fmt;

pub mod boundary_constraints;
pub use boundary_constraints::*;

pub mod transition_constraints;
pub use transition_constraints::*;

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

/// Arithmetic expressions for constraint evaluation.
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Const(u64),
    Var(Identifier),
    Next(Identifier),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Exp(Box<Expr>, u64),
}

/// [Identifier] is used to represent variable names.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct Identifier(pub String);

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}
