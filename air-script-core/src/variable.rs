use super::{Expression, Identifier, ListComprehension};
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VariableBinding {
    name: Identifier,
    value: VariableValueExpr,
}

impl VariableBinding {
    pub fn new(name: Identifier, value: VariableValueExpr) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> &str {
        self.name.name()
    }

    pub fn value(&self) -> &VariableValueExpr {
        &self.value
    }

    pub fn into_parts(self) -> (String, VariableValueExpr) {
        (self.name.into_name(), self.value)
    }
}

/// The expression or expressions that define the value of a variable binding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableValueExpr {
    Scalar(Expression),
    Vector(Vec<Expression>),
    Matrix(Vec<Vec<Expression>>),
    ListComprehension(ListComprehension),
}

impl Display for VariableValueExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scalar(_) => write!(f, "scalar"),
            Self::Vector(_) => write!(f, "vector"),
            Self::Matrix(_) => write!(f, "matrix"),
            Self::ListComprehension(_) => write!(f, "list comprehension"),
        }
    }
}
