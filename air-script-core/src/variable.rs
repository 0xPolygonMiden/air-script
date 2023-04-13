use super::{Expression, Identifier, ListComprehension};
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VariableBinding {
    name: Identifier,
    value: VariableType,
}

impl VariableBinding {
    pub fn new(name: Identifier, value: VariableType) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> &str {
        self.name.name()
    }

    pub fn value(&self) -> &VariableType {
        &self.value
    }

    pub fn into_parts(self) -> (String, VariableType) {
        (self.name.into_name(), self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableType {
    Scalar(Expression),
    Vector(Vec<Expression>),
    Matrix(Vec<Vec<Expression>>),
    ListComprehension(ListComprehension),
}

impl Display for VariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scalar(_) => write!(f, "scalar"),
            Self::Vector(_) => write!(f, "vector"),
            Self::Matrix(_) => write!(f, "matrix"),
            Self::ListComprehension(_) => write!(f, "list comprehension"),
        }
    }
}
