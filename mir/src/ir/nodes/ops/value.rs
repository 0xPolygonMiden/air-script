use air_parser::ast::{BusType, Identifier, QualifiedIdentifier, TraceColumnIndex, TraceSegmentId};
use air_types::*;
use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{BackLink, Builder, BuilderHook, Bus, Child, Link, Node, Op, Owner, Singleton};

/// A MIR operation to represent a known value, [Value].
///
/// Wraps a [SpannedMirValue] to represent a known value in the MIR.
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Op)]
pub struct Value {
    pub parents: Vec<BackLink<Owner>>,
    #[span]
    pub value: SpannedMirValue,
    pub _node: Singleton<Node>,
    pub ty: Option<Type>,
}

impl ScalarTypeMut for Value {
    fn update_scalar_ty_unchecked(&mut self, new_sty: Option<ScalarType>) {
        self.ty.update_scalar_ty_unchecked(new_sty);
    }
}

impl TypeMut for Value {
    fn update_ty_unchecked(&mut self, new_ty: Option<Type>) {
        self.ty = new_ty;
    }
}

impl Typing for Value {
    fn ty(&self) -> Option<Type> {
        self.ty.ty()
    }
}

impl BuilderHook for Value {}

impl Value {
    pub fn create(value: SpannedMirValue) -> Link<Op> {
        Op::Value(Self { value, ..Default::default() }).into()
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self {
            value: SpannedMirValue {
                value: MirValue::Constant(ConstantValue::Scalar(value as u64)),
                span: Default::default(),
            },
            ..Default::default()
        }
    }
}

impl Child for Value {
    type Parent = Owner;
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        self.parents.clone()
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.push(parent.into());
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.retain(|p| *p != parent.clone().into());
    }
}

/// Represents a known value in the MIR.
///
/// Values are either constant, or evaluated at runtime using the context
/// provided to an AirScript program (i.e. public inputs, etc.).
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum MirValue {
    /// A constant value.
    Constant(ConstantValue),
    /// A reference to a specific column in the trace segment, with an optional offset.
    TraceAccess(TraceAccess),
    /// A reference to a periodic column.
    ///
    /// The value this corresponds to is determined by the current row of the trace.
    PeriodicColumn(PeriodicColumnAccess),
    /// A reference to a specific element of a given public input
    PublicInput(PublicInputAccess),
    /// A reference to a public input table.
    PublicInputTable(PublicInputTableAccess),
    /// A reference to a specific index in the random values array.
    ///
    /// Random values are not provided by the user in the AirScript program, but are used to expand
    /// Bus constraints.
    RandomValue(usize),
    /// A binding to a set of consecutive trace columns of a given size.
    TraceAccessBinding(TraceAccessBinding),
    /// A binding to a [Bus].
    BusAccess(BusAccess),
    /// An empty bus
    Null,
    /// An unconstrained bus
    Unconstrained,
}

/// [BusAccess] is like SymbolAccess, but is used to describe an access to a specific bus.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BusAccess {
    /// The trace segment being accessed
    pub bus: Link<Bus>,
    /// The offset from the current row.
    ///
    /// Defaults to 0, which indicates no offset/the current row.
    ///
    /// For example, if accessing a trace column with `a'`, where `a` is bound to a single column,
    /// the row offset would be `1`, as the `'` modifier indicates the "next" row.
    pub row_offset: usize,
}

impl BusAccess {
    /// Creates a new [BusAccess].
    pub const fn new(bus: Link<Bus>, row_offset: usize) -> Self {
        Self { bus, row_offset }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum ConstantValue {
    Scalar(u64),
    Vector(Vec<u64>),
    Matrix(Vec<Vec<u64>>),
}

impl Typing for ConstantValue {
    fn ty(&self) -> Option<Type> {
        match self {
            ConstantValue::Scalar(_) => ty!(uint),
            ConstantValue::Vector(v) => ty!(uint[v.len()]),
            ConstantValue::Matrix(m) => {
                let row_count = m.len();
                if row_count == 0 {
                    return ty!(uint[usize::MAX, usize::MAX]);
                }
                let col_count = m.iter().map(|r| r.len()).max().unwrap_or(usize::MAX);
                ty!(uint[row_count, col_count])
            },
        }
    }
}

/// [TraceAccess] is like SymbolAccess, but is used to describe an access to a specific trace
/// column or columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceAccess {
    /// The trace segment being accessed
    pub segment: TraceSegmentId,
    /// The index of the first column at which the access begins
    pub column: TraceColumnIndex,
    /// The offset from the current row.
    ///
    /// Defaults to 0, which indicates no offset/the current row.
    ///
    /// For example, if accessing a trace column with `a'`, where `a` is bound to a single column,
    /// the row offset would be `1`, as the `'` modifier indicates the "next" row.
    pub row_offset: usize,
    /// The type of the value being accessed, if known.
    /// Defaults to None until the access is resolved.
    /// This should only be a felt or [felt; n] type.
    ty: Option<Type>,
}
impl TraceAccess {
    /// Creates a new [TraceAccess].
    pub const fn new(segment: TraceSegmentId, column: TraceColumnIndex, row_offset: usize) -> Self {
        Self { segment, column, row_offset, ty: None }
    }
}
impl ScalarTypeMut for TraceAccess {
    fn update_scalar_ty_unchecked(&mut self, new_sty: Option<ScalarType>) {
        self.ty.update_scalar_ty_unchecked(new_sty);
    }
}
impl TypeMut for TraceAccess {
    fn update_ty_unchecked(&mut self, new_ty: Option<Type>) {
        self.ty = new_ty;
    }
}
impl Typing for TraceAccess {
    fn ty(&self) -> Option<Type> {
        self.ty.ty()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct TraceAccessBinding {
    pub segment: TraceSegmentId,
    /// The offset to the first column of the segment which is bound by this binding
    pub offset: usize,
    /// The number of columns which are bound
    pub size: usize,
}
impl Typing for TraceAccessBinding {
    fn ty(&self) -> Option<Type> {
        if self.size == 1 {
            ty!(felt)
        } else {
            ty!(felt[self.size])
        }
    }
}

/// Represents a typed value in the MIR.
#[derive(Debug, Eq, PartialEq, Clone, Hash, Spanned)]
pub struct SpannedMirValue {
    #[span]
    pub span: SourceSpan,
    pub value: MirValue,
}

/// Represents an access of a PeriodicColumn, similar in nature to [TraceAccess].
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct PeriodicColumnAccess {
    pub name: QualifiedIdentifier,
    pub cycle: usize,
}
impl PeriodicColumnAccess {
    pub const fn new(name: QualifiedIdentifier, cycle: usize) -> Self {
        Self { name, cycle }
    }
}

/// Represents an access of a PublicInput, similar in nature to [TraceAccess].
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct PublicInputAccess {
    /// The name of the public input to access
    pub name: Identifier,
    /// The index of the element in the public input to access
    pub index: usize,
    /// The type of the value being accessed, if known.
    /// Defaults to None until the access is resolved.
    ty: Option<Type>,
}
impl PublicInputAccess {
    pub const fn new(name: Identifier, index: usize) -> Self {
        Self { name, index, ty: None }
    }
}
impl ScalarTypeMut for PublicInputAccess {
    fn update_scalar_ty_unchecked(&mut self, new_sty: Option<ScalarType>) {
        self.ty.update_scalar_ty_unchecked(new_sty);
    }
}
impl TypeMut for PublicInputAccess {
    fn update_ty_unchecked(&mut self, new_ty: Option<Type>) {
        self.ty = new_ty;
    }
}
impl Typing for PublicInputAccess {
    fn ty(&self) -> Option<Type> {
        self.ty.ty()
    }
}

/// Represents an access of a public input table, similar in nature to [TraceAccess].
///
/// It can only be bound to a [Bus]'s .first or .last boundary constraints.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PublicInputTableAccess {
    /// The name of the public input to bind
    pub table_name: Identifier,
    /// The number of columns in the table
    pub num_cols: usize,
    /// The type of bus to bind (multiset or logUp).
    /// The bus type is not always known at the time of instantiation,
    /// making it an Option allows setting it later.
    bus_type: Option<BusType>,
}

impl PublicInputTableAccess {
    pub const fn new(table_name: Identifier, num_cols: usize) -> Self {
        Self { table_name, num_cols, bus_type: None }
    }
    pub fn set_bus_type(&mut self, bus_type: BusType) {
        self.bus_type = Some(bus_type);
    }
    pub fn bus_type(&self) -> BusType {
        self.bus_type.expect("Bus type should have already been set")
    }
}

impl Default for SpannedMirValue {
    fn default() -> Self {
        Self {
            value: MirValue::Constant(ConstantValue::Scalar(0)),
            span: Default::default(),
        }
    }
}

impl Typing for PublicInputTableAccess {
    fn ty(&self) -> Option<Type> {
        ty!(felt[usize::MAX, self.num_cols])
    }
}
