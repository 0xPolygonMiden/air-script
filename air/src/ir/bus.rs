use air_parser::ast::Identifier;
pub use air_parser::ast::{BusConstraintForm, BusType};
pub use mir::ir::BusOpKind;

use crate::NodeIndex;

/// An Air struct to represent a Bus definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bus {
    /// The [Identifier] of the bus
    pub name: Identifier,
    /// The type of bus:
    pub bus_type: BusType,
    /// Constraint form for multiset buses.
    pub constraint_form: BusConstraintForm,
    /// The initial state of the bus
    pub first: BusBoundary,
    /// Optional tag for the first-row boundary constraint.
    pub first_tag: Option<u64>,
    /// The final state of the bus
    pub last: BusBoundary,
    /// Optional tag for the last-row boundary constraint.
    pub last_tag: Option<u64>,
    /// The operations (insertions and removals) of this bus
    pub bus_ops: Vec<BusOp>,
    /// Optional tag for the bus transition constraint.
    pub transition_tag: Option<u64>,
}

/// Represents the boundaries of a bus, which can be either a public input table or an empty bus.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BusBoundary {
    /// A reference to a public input table.
    PublicInputTable(PublicInputTableAccess),
    /// A reference to an empty bus
    Null,
    /// An unconstrained bus boundary
    Unconstrained,
}

/// Represents an access of a public input table.
///
/// It can only be bound to a [Bus]'s .first or .last boundary constraints.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublicInputTableAccess {
    /// The name of the public input to bind
    pub table_name: Identifier,
    /// The number of columns in the public input table
    pub num_cols: usize,
    /// The type of the bus
    pub bus_type: BusType,
}
impl PublicInputTableAccess {
    pub const fn new(table_name: Identifier, num_cols: usize, bus_type: BusType) -> Self {
        Self { table_name, num_cols, bus_type }
    }
}

/// Represent an operation on a bus, such as inserting or removing values.  
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusOp {
    /// The [NodeIndex] of each value in the tuple being inserted or removed.  
    pub columns: Vec<NodeIndex>,
    // The [NodeIndex] of the selector which defines when this bus operation is activated.
    pub latch: NodeIndex,
    /// The kind of operation (insert or remove).  
    pub op_kind: BusOpKind,
}

impl BusOp {
    pub fn new(columns: Vec<NodeIndex>, latch: NodeIndex, op_kind: BusOpKind) -> Self {
        Self { columns, latch, op_kind }
    }
}

impl Bus {
    pub fn new(
        name: Identifier,
        bus_type: BusType,
        constraint_form: BusConstraintForm,
        first: BusBoundary,
        last: BusBoundary,
        bus_ops: Vec<BusOp>,
        first_tag: Option<u64>,
        last_tag: Option<u64>,
        transition_tag: Option<u64>,
    ) -> Self {
        Self {
            name,
            bus_type,
            constraint_form,
            first,
            last,
            bus_ops,
            first_tag,
            last_tag,
            transition_tag,
        }
    }
}
