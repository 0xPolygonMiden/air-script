use super::{
    ast::{ConstraintExpr, IntegrityStmt},
    AccessType, BTreeMap, ConstantValueExpr, ConstraintBuilder, ConstraintDomain, Expression,
    Identifier, Iterable, ListComprehension, ListFolding, ListFoldingValueExpr, SemanticError,
    Symbol, SymbolAccess, SymbolBinding, VariableBinding, VariableValueExpr, CURRENT_ROW,
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
            IntegrityStmt::Constraint(constraint) => {
                let (constraint_expr, _, selectors) = constraint.into_parts();
                match constraint_expr {
                    ConstraintExpr::Inline(inline_constraint) => {
                        let (lhs, rhs) = inline_constraint.into_parts();
                        // add the left hand side expression to the graph.
                        let lhs = self.insert_expr(lhs)?;

                        // add the right hand side expression to the graph.
                        let rhs = self.insert_expr(rhs)?;

                        // add the selectors expression to the graph
                        let selectors = if let Some(selectors) = selectors {
                            Some(self.insert_expr(selectors)?)
                        } else {
                            None
                        };

                        // merge the two sides of the expression into a constraint.
                        let root = self.merge_equal_exprs(lhs, rhs, selectors);

                        // get the trace segment and domain of the constraint
                        // the default domain for integrity constraints is `EveryRow`
                        let (trace_segment, domain) =
                            self.graph.node_details(&root, ConstraintDomain::EveryRow)?;

                        // save the constraint information
                        self.insert_constraint(root, trace_segment.into(), domain)?;
                    }
                    ConstraintExpr::Evaluator(ev_call) => {
                        self.process_evaluator_call(ev_call)?;
                    }
                }
            }
            IntegrityStmt::VariableBinding(variable) => {
                if let VariableValueExpr::ListComprehension(list_comprehension) = variable.value() {
                    let vector = self.unfold_lc(list_comprehension)?;
                    self.symbol_table.insert_variable(VariableBinding::new(
                        Identifier(variable.name().to_string()),
                        VariableValueExpr::Vector(vector),
                    ))?
                } else {
                    self.symbol_table.insert_variable(variable)?
                }
            }
        }

        Ok(())
    }
}
