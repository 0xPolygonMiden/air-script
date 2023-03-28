use super::{MatrixAccess, TraceAccess, VectorAccess};

#[derive(Debug, Eq, PartialEq)]
pub enum Value {
    /// An inlined or named constant with identifier and access indices.
    Constant(ConstantValue),
    /// An identifier for an element in the trace segment, column, and row offset specified by the
    /// [TraceAccess]
    TraceElement(TraceAccess),
    /// An identifier for a periodic value from a specified periodic column. The first inner value
    /// is the index of the periodic column within the declared periodic columns. The second inner
    /// value is the length of the column's periodic cycle. The periodic value made available from
    /// the specified column is based on the current row of the trace.
    PeriodicColumn(usize, usize),
    /// An identifier for a public input declared by the specified name and accessed at the
    /// specified index.
    PublicInput(String, usize),
    /// A random value provided by the verifier. The inner value is the index of this random value
    /// in the array of all random values.
    RandomValue(usize),
}

#[derive(Debug, Eq, PartialEq)]
pub enum ConstantValue {
    Inline(u64),
    Scalar(String),
    Vector(VectorAccess),
    Matrix(MatrixAccess),
}
