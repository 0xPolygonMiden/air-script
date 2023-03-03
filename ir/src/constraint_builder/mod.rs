use super::{
    ast, AccessType, BTreeMap, BTreeSet, ConstantType, ConstantValue, ConstraintDomain,
    Constraints, Declarations, Expression, Identifier, IndexedTraceAccess, Iterable,
    ListComprehension, ListFoldingType, ListFoldingValueType, MatrixAccess, NamedTraceAccess,
    NodeIndex, Operation, Scope, SemanticError, Symbol, SymbolTable, SymbolType, TraceSegment,
    Value, VariableType, VectorAccess, CURRENT_ROW,
};

mod constrained_boundary;
pub(crate) use constrained_boundary::ConstrainedBoundary;

mod expression;

mod list_comprehension;
mod list_folding;

mod variables;

// CONSTRAINT BUILDER
// ================================================================================================

// TODO: docs
#[derive(Default, Debug)]
pub(super) struct ConstraintBuilder {
    // TODO: docs
    symbol_table: SymbolTable,

    /// A set of all boundaries which have been constrained. This is used to ensure that no more
    /// than one constraint is defined at any given boundary.
    constrained_boundaries: BTreeSet<ConstrainedBoundary>,

    // TODO: docs
    constraints: Constraints,
}

impl ConstraintBuilder {
    pub fn new(symbol_table: SymbolTable) -> Self {
        let constraints = Constraints::new(symbol_table.num_trace_segments());
        Self {
            symbol_table,
            constrained_boundaries: BTreeSet::new(),
            constraints,
        }
    }

    pub fn into_air(self) -> (Declarations, Constraints) {
        (self.symbol_table.into_declarations(), self.constraints)
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// Adds the provided parsed boundary statement to the graph. The statement can either be a
    /// variable defined in the boundary constraints section or a boundary constraint expression.
    ///
    /// In case the statement is a variable, it is added to the symbol table.
    ///
    /// In case the statement is a constraint, the constraint is turned into a subgraph which is
    /// added to the [AlgebraicGraph] (reusing any existing nodes). The index of its entry node
    /// is then saved in the boundary_constraints matrix.
    pub(super) fn insert_boundary_stmt(
        &mut self,
        stmt: ast::BoundaryStmt,
    ) -> Result<(), SemanticError> {
        match stmt {
            ast::BoundaryStmt::Constraint(constraint) => {
                let trace_access = self
                    .symbol_table
                    .get_trace_access_by_name(constraint.access())?;
                let domain = constraint.boundary().into();
                let constrained_boundary = ConstrainedBoundary::new(
                    trace_access.trace_segment(),
                    trace_access.col_idx(),
                    domain,
                );
                // add the boundary to the set of constrained boundaries.
                if !self.constrained_boundaries.insert(constrained_boundary) {
                    // raise an error if the same boundary was previously constrained
                    return Err(SemanticError::boundary_already_constrained(
                        &constrained_boundary,
                    ));
                }

                // add the trace access at the specified boundary to the graph.
                let lhs = self.insert_trace_access(&trace_access)?;

                // add its expression to the constraints graph.
                let rhs = self.insert_expr(constraint.value())?;

                // TODO: check trace segment
                // // ensure that the inferred trace segment of the rhs expression can be applied to
                // // column against which the boundary constraint is applied.
                // // trace segment inference defaults to the lowest segment (the main trace) and is
                // // adjusted according to the use of random values and trace columns.
                // if lhs.trace_segment() < rhs.trace_segment() {
                //     return Err(SemanticError::trace_segment_mismatch(lhs.trace_segment()));
                // }

                // merge the two sides of the expression into a constraint.
                let root = self.merge_equal_exprs(lhs, rhs)?;

                // get the trace segment and domain of the constraint
                let (trace_segment, domain) = self.constraints.node_details(&root, domain)?;

                // save the constraint information
                self.insert_constraint(root, trace_segment, domain)?
            }
            ast::BoundaryStmt::Variable(variable) => {
                self.symbol_table.insert_boundary_variable(variable)?
            }
        }

        Ok(())
    }

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
        stmt: ast::IntegrityStmt,
    ) -> Result<(), SemanticError> {
        match stmt {
            ast::IntegrityStmt::Constraint(constraint) => {
                // add the left hand side expression to the graph.
                let lhs = self.insert_expr(constraint.lhs())?;

                // add the right hand side expression to the graph.
                let rhs = self.insert_expr(constraint.rhs())?;

                // merge the two sides of the expression into a constraint.
                let root = self.merge_equal_exprs(lhs, rhs)?;

                // get the trace segment and domain of the constraint
                // the default domain for integrity constraints is `EveryRow`
                let (trace_segment, domain) = self
                    .constraints
                    .node_details(&root, ConstraintDomain::EveryRow)?;

                // save the constraint information
                self.insert_constraint(root, trace_segment, domain)
            }
            ast::IntegrityStmt::Variable(variable) => {
                let (name, variable_type) = variable.into_parts();

                match variable_type {
                    VariableType::ListComprehension(list_comprehension) => {
                        let vector = self.unfold_lc(&list_comprehension)?;
                        self.symbol_table
                            .insert_integrity_variable(name, VariableType::Vector(vector))
                    }
                    _ => self
                        .symbol_table
                        .insert_integrity_variable(name, variable_type),
                }
            }
        }
    }

    /// Takes two expressions which are expected to be equal and merges them into a constraint (a
    /// subtree in the graph that must be equal to zero for a particular domain). The constraint is
    /// then saved in the appropriate constraint list (boundary, validity, or transition).
    fn insert_constraint(
        &mut self,
        root: NodeIndex,
        trace_segment: TraceSegment,
        domain: ConstraintDomain,
    ) -> Result<(), SemanticError> {
        // TODO: validate trace segment
        let trace_segment = trace_segment.into();

        // the constraint should not be against an undeclared trace segment.
        if self.symbol_table.num_trace_segments() <= trace_segment {
            return Err(SemanticError::InvalidConstraint(
                "Constraint against undeclared trace segment".to_string(),
            ));
        }

        // add the constraint to the constraints
        self.constraints
            .insert_constraint(root, trace_segment, domain);

        Ok(())
    }
}
