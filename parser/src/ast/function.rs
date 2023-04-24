use air_script_core::{VariableValueExpr, Expression};

use super::{Identifier, IntegrityStmt};

#[derive(Debug, Eq, PartialEq)]
pub struct Function {
    name: Identifier,
    params: Vec<FunctionParam>,
    return_type: Vec<ValueType>,
    body: FunctionBody,
}

impl Function {
    /// Creates a new function.
    pub fn new(
        name: Identifier,
        params: Vec<FunctionParam>,
        return_type: Vec<ValueType>,
        body: FunctionBody,
    ) -> Self {
        Self {
            name,
            params,
            return_type,
            body,
        }
    }

    /// Returns the name of the function.
    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// Returns the arguments of the function.
    pub fn arguments(&self) -> &Vec<FunctionParam> {
        &self.arguments
    }

    /// Returns the return type of the function.
    pub fn return_type(&self) -> &Vec<ValueType> {
        &self.return_type
    }

    /// Returns the body of the function.
    pub fn body(&self) -> &FunctionBody {
        &self.body
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FunctionParam {
    name: Identifier,
    argument_type: ValueType,
}

impl FunctionParam {
    /// Creates a new function argument.
    pub fn new(name: Identifier, argument_type: ValueType) -> Self {
        Self {
            name,
            argument_type,
        }
    }

    /// Returns the name of the function argument.
    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// Returns the type of the function argument.
    pub fn argument_type(&self) -> &ValueType {
        &self.argument_type
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueType {
    Scalar,
    Vector(u64),
    Matrix(u64, u64),
}

#[derive(Debug, Eq, PartialEq)]
pub struct FunctionBody {
    variables: Vec<IntegrityStmt>,
    return_stmt: Vec<VariableValueExpr>,
}

impl FunctionBody {
    /// Creates a new function body.
    pub fn new(variables: Vec<IntegrityStmt>, return_stmt: Vec<VariableValueExpr>) -> Self {
        Self {
            variables,
            return_stmt,
        }
    }

    /// Returns the variables declared in the function body.
    pub fn variables(&self) -> &Vec<IntegrityStmt> {
        &self.variables
    }

    /// Returns the return statement of the function body.
    pub fn return_stmt(&self) -> &Vec<VariableValueExpr> {
        &self.return_stmt
    }
}

pub enum FunctionArgument {
    Scalar(Expression),
    Vector(Vec<Expression>),
    Matrix(Vec<Vec<Expression>>),
}