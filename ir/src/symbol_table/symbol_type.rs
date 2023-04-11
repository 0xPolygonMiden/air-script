use super::{ConstantValueExpr, TraceBinding, VariableValueExpr};
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum SymbolType {
    /// an identifier for a constant, containing its type and value
    ConstantBinding(ConstantValueExpr),
    /// an identifier for a binding to one or more trace columns, containing the trace binding
    /// information with its identifier, trace segment, size, and offset.
    TraceBinding(TraceBinding),
    /// an identifier for a public input, containing the size of the public input array
    PublicInput(usize),
    /// an identifier for a periodic column, containing its index out of all periodic columns and
    /// its cycle length in that order.
    PeriodicColumn(usize, usize),
    /// an expression or set of expressions associated with a variable
    VariableBinding(VariableValueExpr),
    /// an identifier for random value, containing its index in the random values array and its
    /// length if this value is an array. For non-array random values second parameter is always 1.
    RandomValuesBinding(usize, usize),
}

impl Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConstantBinding(_) => write!(f, "ConstantBinding"),
            Self::TraceBinding(binding) => {
                write!(f, "TraceBinding in segment {}", binding.trace_segment())
            }
            Self::PublicInput(_) => write!(f, "PublicInput"),
            Self::PeriodicColumn(_, _) => write!(f, "PeriodicColumn"),
            Self::VariableBinding(_) => write!(f, "VariableBinding"),
            Self::RandomValuesBinding(_, _) => write!(f, "RandomValuesBinding"),
        }
    }
}
