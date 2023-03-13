use super::{
    ast::IntegrityStmt, BTreeMap, ConstantType, ConstraintBuilder, ConstraintDomain, Expression,
    Identifier, IndexedTraceAccess, Iterable, ListComprehension, ListFoldingType,
    ListFoldingValueType, NamedTraceAccess, SemanticError, Symbol, SymbolType, Variable,
    VariableType, VectorAccess, CURRENT_ROW,
};
// TODO: Move this to air-script-core
use parser::ast::ConstraintType;

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
    /// is then saved in the validity_constraints or transition_constraints matrices.
    pub(super) fn insert_integrity_stmt(
        &mut self,
        stmt: IntegrityStmt,
    ) -> Result<(), SemanticError> {
        match stmt {
            IntegrityStmt::Constraint(ConstraintType::Inline(constraint), _) => {
                // add the left hand side expression to the graph.
                let lhs = self.insert_expr(constraint.lhs())?;

                // add the right hand side expression to the graph.
                let rhs = self.insert_expr(constraint.rhs())?;

                // merge the two sides of the expression into a constraint.
                let root = self.merge_equal_exprs(lhs, rhs);

                // get the trace segment and domain of the constraint
                // the default domain for integrity constraints is `EveryRow`
                let (trace_segment, domain) = self
                    .constraints
                    .node_details(&root, ConstraintDomain::EveryRow)?;

                // save the constraint information
                self.insert_constraint(root, trace_segment.into(), domain)?;
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
            IntegrityStmt::Constraint(ConstraintType::Evaluator(_), _) => todo!(),
        }

        Ok(())
    }
}
