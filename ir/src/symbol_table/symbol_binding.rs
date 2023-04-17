use super::{ConstantValueExpr, TraceBinding, VariableValueExpr};
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum SymbolBinding {
    /// an identifier for a constant, containing its type and value
    Constant(ConstantValueExpr),
    /// an identifier for a binding to one or more trace columns, containing the trace binding
    /// information with its identifier, trace segment, size, and offset.
    Trace(TraceBinding),
    /// an identifier for a public input, containing the size of the public input array
    PublicInput(usize),
    /// an identifier for a periodic column, containing its index out of all periodic columns and
    /// its cycle length in that order.
    PeriodicColumn(usize, usize),
    /// an expression or set of expressions associated with a variable
    Variable(VariableValueExpr),
    /// an identifier for random value, containing its index in the random values array and its
    /// length if this value is an array. For non-array random values second parameter is always 1.
    RandomValues(usize, usize),
}

impl Display for SymbolBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(_) => write!(f, "Constant"),
            Self::Trace(binding) => {
                write!(f, "Trace in segment {}", binding.trace_segment())
            }
            Self::PublicInput(_) => write!(f, "PublicInput"),
            Self::PeriodicColumn(_, _) => write!(f, "PeriodicColumn"),
            Self::Variable(_) => write!(f, "Variable"),
            Self::RandomValues(_, _) => write!(f, "RandomValues"),
        }
    }
}
