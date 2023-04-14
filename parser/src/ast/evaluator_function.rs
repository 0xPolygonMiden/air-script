use super::{Identifier, IntegrityStmt, SymbolAccess, TraceBinding};

/// Evaluator functions take a vector of trace bindings as parameters where each trace binding
/// represents one or a group of columns in the execution trace that are passed to the evaluator
/// function, and enforce integrity constraints on those trace columns.
#[derive(Debug, Eq, PartialEq)]
pub struct EvaluatorFunction {
    name: Identifier,
    params: Vec<Vec<TraceBinding>>,
    integrity_stmts: Vec<IntegrityStmt>,
}

impl EvaluatorFunction {
    /// Creates a new function.
    pub fn new(
        name: Identifier,
        params: Vec<Vec<TraceBinding>>,
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

    /// Returns the integrity statements of the evaluator function.
    pub fn integrity_stmts(&self) -> &[IntegrityStmt] {
        &self.integrity_stmts
    }

    /// Returns the name, trace bindings and integrity statements of the evaluator function.
    pub fn into_parts(self) -> (String, Vec<Vec<TraceBinding>>, Vec<IntegrityStmt>) {
        (self.name.into_name(), self.params, self.integrity_stmts)
    }
}

/// Evaluator function call is used to invoke an evaluator function. It takes a vector of vectors
/// of trace binding accesses as input, where each vector of trace binding accesses represents
/// trace columns of that trace segment that are used as arguments to the evaluator function.
#[derive(Debug, Eq, PartialEq)]
pub struct EvaluatorFunctionCall {
    name: Identifier,
    args: Vec<Vec<SymbolAccess>>,
}

impl EvaluatorFunctionCall {
    /// Creates a new evaluator function call.
    pub fn new(name: Identifier, args: Vec<Vec<SymbolAccess>>) -> Self {
        Self { name, args }
    }

    /// Returns the name of the evaluator function.
    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// Returns the arguments of the evaluator function.
    pub fn args(&self) -> &Vec<Vec<SymbolAccess>> {
        &self.args
    }

    /// Returns the name and arguments of the evaluator function.
    pub fn into_parts(self) -> (String, Vec<Vec<SymbolAccess>>) {
        (self.name.into_name(), self.args)
    }
}
