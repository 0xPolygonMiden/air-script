use super::{Identifier, ListFolding, SymbolAccess, TraceAccess, TraceBindingAccess};

/// Arithmetic expressions for evaluation of constraints.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression {
    Const(u64),
    /// Represents a reference to all or part of a constant, variable, or trace binding.
    SymbolAccess(SymbolAccess),
    TraceAccess(TraceAccess),
    TraceBindingAccess(TraceBindingAccess),
    /// Represents a random value provided by the verifier. The first inner value is the name of
    /// the random values array and the second is the index of this random value in that array
    Rand(Identifier, usize),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Exp(Box<Expression>, Box<Expression>),
    ListFolding(ListFolding),
}
