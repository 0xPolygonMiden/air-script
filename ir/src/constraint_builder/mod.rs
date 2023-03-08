use super::{
    ast, BTreeMap, BTreeSet, ConstantType, ConstrainedBoundary, ConstraintDomain, Constraints,
    Declarations, Expression, Identifier, IdentifierType, IndexedTraceAccess, Iterable,
    ListComprehension, ListFoldingType, ListFoldingValueType, NamedTraceAccess, NodeIndex, Scope,
    SemanticError, SymbolTable, Variable, VariableType, VectorAccess, CURRENT_ROW,
};

mod list_comprehension;
// TODO: get rid of the need to make this public
pub(crate) use list_comprehension::unfold_lc;

mod list_folding;
pub(crate) use list_folding::build_list_from_list_folding_value;

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
                let lhs = self
                    .constraints
                    .insert_trace_access(&self.symbol_table, &trace_access)?;

                // get the trace segment and domain of the boundary column access
                let (lhs_segment, lhs_domain) = self.constraints.node_details(&lhs, domain)?;
                debug_assert!(
                   lhs_domain == domain,
                   "The boundary constraint's domain should be {lhs_domain:?}, but the domain {domain:?} was inferred by the graph",
               );

                // add its expression to the constraints graph.
                let rhs =
                    self.constraints
                        .insert_expr(&self.symbol_table, constraint.value(), domain)?;
                // get the trace segment and domain of the expression
                let (rhs_segment, rhs_domain) = self.constraints.node_details(&rhs, domain)?;

                // ensure that the inferred trace segment and domain of the rhs expression can be
                // applied to column against which the boundary constraint is applied.
                if lhs_segment < rhs_segment {
                    // trace segment inference defaults to the lowest segment (the main trace) and is
                    // adjusted according to the use of random values and trace columns.
                    return Err(SemanticError::trace_segment_mismatch(lhs_segment));
                }
                if lhs_domain != rhs_domain {
                    return Err(SemanticError::incompatible_constraint_domains(
                        &lhs_domain,
                        &rhs_domain,
                    ));
                }

                // merge the two sides of the expression into a constraint.
                let root = self.constraints.merge_equal_exprs(lhs, rhs);

                // save the constraint information
                self.insert_constraint(root, lhs_segment.into(), domain)?
            }
            ast::BoundaryStmt::Variable(variable) => self
                .symbol_table
                .insert_variable(Scope::BoundaryConstraints, variable)?,
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
                let lhs = self.constraints.insert_expr(
                    &self.symbol_table,
                    constraint.lhs(),
                    ConstraintDomain::EveryRow,
                )?;

                // add the right hand side expression to the graph.
                let rhs = self.constraints.insert_expr(
                    &self.symbol_table,
                    constraint.rhs(),
                    ConstraintDomain::EveryRow,
                )?;

                // merge the two sides of the expression into a constraint.
                let root = self.constraints.merge_equal_exprs(lhs, rhs);

                // get the trace segment and domain of the constraint
                // the default domain for integrity constraints is `EveryRow`
                let (trace_segment, domain) = self
                    .constraints
                    .node_details(&root, ConstraintDomain::EveryRow)?;

                // save the constraint information
                self.insert_constraint(root, trace_segment.into(), domain)?;
            }
            ast::IntegrityStmt::Variable(variable) => {
                if let VariableType::ListComprehension(list_comprehension) = variable.value() {
                    let vector = unfold_lc(list_comprehension, &self.symbol_table)?;
                    self.symbol_table.insert_variable(
                        Scope::IntegrityConstraints,
                        Variable::new(
                            Identifier(variable.name().to_string()),
                            VariableType::Vector(vector),
                        ),
                    )?
                } else {
                    self.symbol_table
                        .insert_variable(Scope::IntegrityConstraints, variable)?
                }
            }
        }

        Ok(())
    }

    /// Takes two expressions which are expected to be equal and merges them into a constraint (a
    /// subtree in the graph that must be equal to zero for a particular domain). The constraint is
    /// then saved in the appropriate constraint list (boundary, validity, or transition).
    fn insert_constraint(
        &mut self,
        root: NodeIndex,
        trace_segment: usize,
        domain: ConstraintDomain,
    ) -> Result<(), SemanticError> {
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
