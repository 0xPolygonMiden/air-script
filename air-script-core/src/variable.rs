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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

/// Contains values to be iterated over in a list comprehension.
///
/// For e.g. in the list comprehension \[x + y + z for (x, y, z) in (x, 0..5, z\[1..6\])\],
/// `x` is an Iterable of type Identifier representing the vector to iterate over,
/// `0..5` is an Iterable of type Range representing the range to iterate over,
/// `z[1..6]` is an Iterable of type Slice representing the slice of the vector z to iterate over.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Iterable {
    Identifier(Identifier),
    Range(Range),
    Slice(Identifier, Range),
}
