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
    ///
    TraceAccess(TraceAccess),
    /// A reference to a specific variable in a function
    /// Variable(MirType, argument position, function index)
    Variable(MirType, usize, Function),
    /// A function definition
    /// Definition(arguments, return type, body)
    Definition(Vec<Parameter>, Parameter, Block),
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
    /// We should delete the <TraceAccess> and <RandomValue> variants if we decide to use only the most generic variants below
    TraceAccessBinding(TraceAccessBinding),
    ///RandomValueBinding is a binding to a range of random values
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

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MirType {
    Felt,
    Vector(usize),
    Matrix(usize, usize),
    Definition(Vec<usize>, usize),
}

impl MirValue {
    /*fn ty(&self) -> MirType {
        match &self {
            MirValue::Constant(c) => match c {
                ConstantValue::Felt(_) => MirType::Felt,
                ConstantValue::Vector(v) => MirType::Vector(v.len()),
                ConstantValue::Matrix(m) => MirType::Matrix(m.len(), m[0].len()),
            },
            MirValue::TraceAccess(_) => MirType::Felt,
            MirValue::PeriodicColumn(_) => MirType::Felt,
            MirValue::PublicInput(_) => MirType::Felt,
            MirValue::RandomValue(_) => MirType::Felt,
            MirValue::TraceAccessBinding(trace_access_binding) => {
                let size = trace_access_binding.size;
                match size {
                    1 => MirType::Felt,
                    _ => MirType::Vector(size),
                }
            },
            MirValue::RandomValueBinding(random_value_binding) =>  {
                let size = random_value_binding.size;
                match size {
                    1 => MirType::Felt,
                    _ => MirType::Vector(size),
                }
            },
            MirValue::Vector(vec) => {
                let size = vec.len();
                let inner_ty = vec[0].ty();
                match inner_ty {
                    MirType::Felt => MirType::Vector(size),
                    MirType::Vector(inner_size) => MirType::Matrix(size, inner_size),
                    MirType::Matrix(_, _) => unreachable!(),
                }
            },
            MirValue::Matrix(vec) => {
                let size = vec.len();
                let inner_size = vec[0].len();
                MirType::Matrix(size, inner_size)
            },
        }
    }*/
}

impl SpannedMirValue {
    /*fn ty(&self) -> MirType {
        self.value.ty()
    }*/
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
