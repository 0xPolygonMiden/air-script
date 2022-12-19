use super::{ConstantType, TraceColumn};
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum IdentifierType {
    /// an identifier for a constant, containing it's type and value
    Constant(ConstantType),
    /// an identifier for a trace column, containing trace column information with its trace segment
    /// and the index of the column in that segment.
    TraceColumn(TraceColumn),
    /// an identifier for a public input, containing the size of the public input array
    PublicInput(usize),
    /// an identifier for a periodic column, containing its index out of all periodic columns and
    /// its cycle length in that order.
    PeriodicColumn(usize, usize),
}

impl Display for IdentifierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(_) => write!(f, "Constant"),
            Self::PublicInput(_) => write!(f, "PublicInput"),
            Self::PeriodicColumn(_, _) => write!(f, "PeriodicColumn"),
            Self::TraceColumn(column) => {
                write!(f, "TraceColumn in segment {}", column.trace_segment())
            }
        }
    }
}
