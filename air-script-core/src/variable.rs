use super::{Expression, Identifier};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variable {
    name: Identifier,
    value: VariableType,
}

impl Variable {
    pub fn new(name: Identifier, value: VariableType) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> &str {
        self.name.name()
    }

    pub fn value(&self) -> &VariableType {
        &self.value
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VariableType {
    Scalar(Expression),
    Vector(Vec<Expression>),
    Matrix(Vec<Vec<Expression>>),
}
