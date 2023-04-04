use air_script_core::VariableType;

use super::{Identifier, IntegrityStmt};

#[derive(Debug, Eq, PartialEq)]
pub struct Function {
    name: Identifier,
    arguments: Vec<FunctionArgument>,
    return_type: Vec<ReturnType>,
    body: FunctionBody,
}

impl Function {
    /// Creates a new function.
    pub fn new(
        name: Identifier,
        arguments: Vec<FunctionArgument>,
        return_type: Vec<ReturnType>,
        body: FunctionBody,
    ) -> Self {
        Self {
            name,
            arguments,
            return_type,
            body,
        }
    }

    /// Returns the name of the function.
    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// Returns the arguments of the function.
    pub fn arguments(&self) -> &Vec<FunctionArgument> {
        &self.arguments
    }

    /// Returns the return type of the function.
    pub fn return_type(&self) -> &Vec<ReturnType> {
        &self.return_type
    }

    /// Returns the body of the function.
    pub fn body(&self) -> &FunctionBody {
        &self.body
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FunctionArgument {
    name: Identifier,
    argument_type: ArgumentType,
}

impl FunctionArgument {
    /// Creates a new function argument.
    pub fn new(name: Identifier, argument_type: ArgumentType) -> Self {
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
    pub fn argument_type(&self) -> &ArgumentType {
        &self.argument_type
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ArgumentType {
    Scalar,
    Vector(u64),
    Matrix(u64, u64),
}

pub type ReturnType = ArgumentType;

#[derive(Debug, Eq, PartialEq)]
pub struct FunctionBody {
    variables: Vec<IntegrityStmt>,
    return_stmt: Vec<VariableType>,
}

impl FunctionBody {
    /// Creates a new function body.
    pub fn new(variables: Vec<IntegrityStmt>, return_stmt: Vec<VariableType>) -> Self {
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
    pub fn return_stmt(&self) -> &Vec<VariableType> {
        &self.return_stmt
    }
}
