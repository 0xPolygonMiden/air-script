mod access;
pub use access::{MatrixAccess, Range, VectorAccess};

mod constant;
pub use constant::{Constant, ConstantType};

mod expression;
pub use expression::{Expression, ListFoldingType, ListFoldingValueType};

mod identifier;
pub use identifier::Identifier;

mod trace;
pub use trace::{
    ColumnGroup, TraceAccess, TraceBinding, TraceBindingAccess, TraceBindingAccessSize,
    TraceSegment,
};

mod variable;
pub use variable::{Iterable, ListComprehension, Variable, VariableType};

// TYPES
// ================================================================================================
pub type ComprehensionContext = Vec<(Identifier, Iterable)>;
