use super::{ColumnGroup, Identifier, IntegrityStmt, TraceBindingAccess};

/// Evaluator functions take column groups as parameters where each column group is a set of
/// columns in the particular trace segment that are passed to the evaluator function, and enforce
/// integrity constraints on those trace columns.
#[derive(Debug, Eq, PartialEq)]
pub struct EvaluatorFunction {
    name: Identifier,
    params: Vec<ColumnGroup>,
    integrity_stmts: Vec<IntegrityStmt>,
}

impl EvaluatorFunction {
    /// Creates a new function.
    pub fn new(
        name: Identifier,
        params: Vec<ColumnGroup>,
        integrity_stmts: Vec<IntegrityStmt>,
    ) -> Self {
        Self {
            name,
            params,
            integrity_stmts,
        }
    }

    /// Returns the name of the evaluator function.
    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// Returns the parameters of the evaluator function.
    pub fn params(&self) -> &[ColumnGroup] {
        &self.params
    }

    /// Returns the integrity statements of the evaluator function.
    pub fn integrity_stmts(&self) -> &[IntegrityStmt] {
        &self.integrity_stmts
    }

    /// Returns the name, main trace columns, auxiliary trace columns, and integrity statements
    /// of the evaluator function.
    pub fn into_parts(self) -> (String, Vec<ColumnGroup>, Vec<IntegrityStmt>) {
        (
            self.name.into_name(),
            self.params,
            self.integrity_stmts,
        )
    }
}

/// Evaluator function call is used to invoke an evaluator function. It takes a vector of vectors of 
/// trace binding accesses as input, where each vector of trace binding accesses represents trace
/// columns of that trace segment that are used as arguments to the evaluator function.
#[derive(Debug, Eq, PartialEq)]
pub struct EvaluatorFunctionCall {
    name: Identifier,
    args: Vec<Vec<TraceBindingAccess>>,
}

impl EvaluatorFunctionCall {
    /// Creates a new evaluator function call.
    pub fn new(name: Identifier, args: Vec<Vec<TraceBindingAccess>>) -> Self {
        Self { name, args }
    }

    /// Returns the name of the evaluator function.
    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// Returns the arguments of the evaluator function.
    pub fn args(&self) -> &Vec<Vec<TraceBindingAccess>> {
        &self.args
    }

    /// Returns the name and arguments of the evaluator function.
    pub fn into_parts(self) -> (String, Vec<Vec<TraceBindingAccess>>) {
        (self.name.into_name(), self.args)
    }
}
