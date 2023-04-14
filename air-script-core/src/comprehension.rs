use super::{Expression, Identifier, Iterable};

// TYPES
// ================================================================================================
pub type ComprehensionContext = Vec<(Identifier, Iterable)>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ListComprehension {
    expression: Box<Expression>,
    context: ComprehensionContext,
}

impl ListComprehension {
    /// Creates a new list comprehension.
    pub fn new(expression: Expression, context: ComprehensionContext) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListFolding {
    Sum(ListFoldingValueExpr),
    Prod(ListFoldingValueExpr),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListFoldingValueExpr {
    Identifier(Identifier),
    Vector(Vec<Expression>),
    ListComprehension(ListComprehension),
}
