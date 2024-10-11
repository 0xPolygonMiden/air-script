use super::*;

/// Represents a scalar value in the [MIR]
///
/// Values are either constant, or evaluated at runtime using the context
/// provided to an AirScript program (i.e. random values, public inputs, etc.).
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MirValue {
    /// A constant value.
    Constant(ConstantValue),
    /// Following to update from the ast::BindingType enum
    /// Goal: To represent the different types of values that can be stored in the MIR
    /// (Scalar, vectors and matrices)
    /// 
    /// A reference to a specific column in the trace segment, with an optional offset.
    TraceAccess(TraceAccess),
    /// A reference to a periodic column
    ///
    /// The value this corresponds to is determined by the current row of the trace.
    PeriodicColumn(PeriodicColumnAccess),
    /// A reference to a specific element of a given public input
    PublicInput(PublicInputAccess),
    /// A reference to the `random_values` array, specifically the element at the given index
    RandomValue(usize),

    /// Vector version of the above, if needed 
    /// (basically the same but with a size argument to allow for continuous access)
    /// 
    TraceAccessBinding(TraceAccessBinding),
    /// 
    RandomValueBinding(RandomValueBinding),

    /// Not sure if the following is needed, would be useful if we want to handle e.g. function call arguments with a single node?
    Vector(Vec<MirValue>),
    Matrix(Vec<Vec<MirValue>>),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ConstantValue {
    Felt(u64),
    Vector(Vec<u64>),
    Matrix(Vec<Vec<u64>>),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TraceAccessBinding {
    pub segment: TraceSegmentId,
    /// The offset to the first column of the segment which is bound by this binding
    pub offset: usize,
    /// The number of columns which are bound
    pub size: usize,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RandomValueBinding {
    /// The offset in the random values array where this binding begins
    pub offset: usize,
    /// The number of elements which are bound
    pub size: usize,
}

/// Represents a typed value in the [MIR]
///
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SpannedMirValue {
    pub span: SourceSpan,
    pub value: MirValue,
}

pub enum MirType {
    Felt,
    Vector(usize),
    Matrix(usize, usize),
}

impl SpannedMirValue {

    fn ty(&self) -> MirType {
        match &self.value {
            MirValue::Constant(c) => match c {
                ConstantValue::Felt(_) => MirType::Felt,
                ConstantValue::Vector(v) => MirType::Vector(v.len()),
                ConstantValue::Matrix(m) => MirType::Matrix(m.len(), m[0].len()),
            },
            MirValue::TraceAccess(t) => MirType::Felt,
            MirValue::PeriodicColumn(p) => MirType::Felt,
            MirValue::PublicInput(p) => MirType::Felt,
            MirValue::RandomValue(_) => MirType::Felt,

        }
    }
}


/// Represents an access of a [PeriodicColumn], similar in nature to [TraceAccess]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PeriodicColumnAccess {
    pub name: QualifiedIdentifier,
    pub cycle: usize,
}
impl PeriodicColumnAccess {
    pub const fn new(name: QualifiedIdentifier, cycle: usize) -> Self {
        Self { name, cycle }
    }
}

/// Represents an access of a [PublicInput], similar in nature to [TraceAccess]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PublicInputAccess {
    /// The name of the public input to access
    pub name: Identifier,
    /// The index of the element in the public input to access
    pub index: usize,
}
impl PublicInputAccess {
    pub const fn new(name: Identifier, index: usize) -> Self {
        Self { name, index }
    }
}
