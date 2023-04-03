use super::{
    ast::{ConstraintType, IntegrityStmt},
    BTreeMap, ConstantType, ConstraintBuilder, Expression, Identifier, Iterable, ListComprehension,
    ListFoldingType, ListFoldingValueType, SemanticError, Symbol, SymbolType, TraceAccess,
    TraceBindingAccess, TraceBindingAccessSize, Variable, VariableType, VectorAccess, CURRENT_ROW,
};

mod list_comprehension;
mod list_folding;

impl ConstraintBuilder {
    /// Adds the provided parsed integrity statement to the graph. The statement can either be a
    /// variable defined in the integrity constraints section or an integrity constraint.
    ///
    /// In case the statement is a variable, it is added to the symbol table.
    ///
    /// In case the statement is a constraint, the constraint is turned into a subgraph which is
    /// added to the [AlgebraicGraph] (reusing any existing nodes). The index of its entry node
    /// is then saved in the integrity_constraints matrix.
    pub(super) fn process_integrity_stmt(
        &mut self,
        stmt: IntegrityStmt,
    ) -> Result<(), SemanticError> {
        match stmt {
            IntegrityStmt::Constraint(ConstraintType::Inline(constraint), _) => {
                let (lhs, rhs) = constraint.into_parts();
                self.insert_constraint(lhs, rhs)?;
            }
            IntegrityStmt::Variable(variable) => {
                if let VariableType::ListComprehension(list_comprehension) = variable.value() {
                    let vector = self.unfold_lc(list_comprehension)?;
                    self.symbol_table.insert_variable(Variable::new(
                        Identifier(variable.name().to_string()),
                        VariableType::Vector(vector),
                    ))?
                } else {
                    self.symbol_table.insert_variable(variable)?
                }
            }
            IntegrityStmt::Constraint(ConstraintType::Evaluator(ev_call), _) => {
                let (name, args) = ev_call.into_parts();

                // ensure the evaluator exists
                let evaluator = self.evaluators.get(&name);
                match evaluator {
                    Some(evaluator) => {
                        // check the arguments against the symbol table and turn them into [TraceAccess].
                        let mut accesses = Vec::new();
                        for segment in args.into_iter() {
                            for binding in segment.into_iter() {
                                let access =
                                    self.symbol_table.get_trace_binding_access(&binding)?;
                                accesses.push(access);
                            }
                        }
                        // apply the evaluator to the arguments and return the resulting graph
                        let (_subgraph, _constraint_nodes) = evaluator.apply(accesses)?;

                        // TODO: insert the subgraph into the main graph and save the entry node index
                    }
                    None => {
                        todo!("Error");
                    }
                }
            }
            IntegrityStmt::ConstraintComprehension(_, _, _) => todo!(),
        }

        Ok(())
    }
}
