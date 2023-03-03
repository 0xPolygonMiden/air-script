use super::{ConstantType, TraceColumns, VariableType};
use std::fmt::Display;

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
    /// an expression or set of expressions associated with a variable
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
