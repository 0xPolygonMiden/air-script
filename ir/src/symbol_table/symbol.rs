use crate::constraints::ConstraintDomain;

use super::{ConstantType, TraceColumns, VariableType};
use std::fmt::Display;

/// Symbol information for a constant, variable, trace column, periodic column, or public input.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Symbol {
    name: String,
    scope: Scope,
    symbol_type: SymbolType,
}

impl Symbol {
    pub(super) fn new(name: String, scope: Scope, symbol_type: SymbolType) -> Self {
        Self {
            name,
            scope,
            symbol_type,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn symbol_type(&self) -> &SymbolType {
        &self.symbol_type
    }
}

/// The scope where an associated element can be used.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum Scope {
    BoundaryConstraints,
    IntegrityConstraints,
    Global,
}

impl From<ConstraintDomain> for Scope {
    fn from(domain: ConstraintDomain) -> Self {
        match domain {
            ConstraintDomain::FirstRow | ConstraintDomain::LastRow => Self::BoundaryConstraints,
            ConstraintDomain::EveryRow | ConstraintDomain::EveryFrame(_) => {
                Self::IntegrityConstraints
            }
        }
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BoundaryConstraints => write!(f, "boundary constraints scope"),
            Self::IntegrityConstraints => write!(f, "integrity constraints scope"),
            Self::Global => write!(f, "global scope"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum SymbolType {
    /// an identifier for a constant, containing its type and value
    Constant(ConstantType),
    /// an identifier for a trace column, containing trace column information with its trace
    /// segment, its size and its offset.
    TraceColumns(TraceColumns),
    /// an identifier for a public input, containing the size of the public input array
    PublicInput(usize),
    /// an identifier for a periodic column, containing its index out of all periodic columns and
    /// its cycle length in that order.
    PeriodicColumn(usize, usize),
    /// an identifier for a variable, containing its scope (boundary or integrity), name, and value
    Variable(VariableType),
    /// an identifier for random value, containing its index in the random values array and its
    /// length if this value is an array. For non-array random values second parameter is always 1.
    RandomValuesBinding(usize, usize),
}

impl Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(_) => write!(f, "Constant"),
            Self::PublicInput(_) => write!(f, "PublicInput"),
            Self::PeriodicColumn(_, _) => write!(f, "PeriodicColumn"),
            Self::TraceColumns(columns) => {
                write!(f, "TraceColumns in segment {}", columns.trace_segment())
            }
            Self::Variable(_) => write!(f, "Variable"),
            Self::RandomValuesBinding(_, _) => write!(f, "RandomValuesBinding"),
        }
    }
}
