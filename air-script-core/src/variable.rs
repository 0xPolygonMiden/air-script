use super::{Expression, Identifier, Range};

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
    Tuple(Vec<Expression>),
    ListComprehension(ListComprehension),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ListComprehension {
    expression: Box<Expression>,
    context: Vec<(Identifier, Iterable)>,
}

impl ListComprehension {
    /// Creates a new list comprehension.
    pub fn new(expression: Expression, context: Vec<(Identifier, Iterable)>) -> Self {
        Self {
            expression: Box::new(expression),
            context,
        }
    }

    /// Returns the expression that is evaluated for each member of the list.
    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    /// Returns the context of the list comprehension.
    pub fn context(&self) -> &[(Identifier, Iterable)] {
        &self.context
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Iterable {
    Identifier(Identifier),
    Range(Range),
    Slice(Identifier, Range),
}
