use super::{
    ast, BTreeMap, BTreeSet, ConstantType, ConstrainedBoundary, ConstraintDomain, Constraints,
    Declarations, Expression, Identifier, IndexedTraceAccess, Iterable, ListComprehension,
    ListFoldingType, ListFoldingValueType, NamedTraceAccess, NodeIndex, Scope, SemanticError,
    Symbol, SymbolAccess, SymbolTable, SymbolType, TraceSegment, Variable, VariableType,
    VectorAccess, CURRENT_ROW,
};

mod expression_details;
// TODO: get rid of the need to make this public
pub(crate) use expression_details::ExprDetails;

mod list_comprehension;
// TODO: get rid of the need to make this public
pub(crate) use list_comprehension::unfold_lc;

mod list_folding;
pub(crate) use list_folding::build_list_from_list_folding_value;

// TYPES
// ================================================================================================

pub(crate) type VariableRoots = BTreeMap<SymbolAccess, ExprDetails>;

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

    /// Variable roots for the variables used in integrity constraints. For each element in a
    /// vector or a matrix, a new root is added with a key equal to the [VariableValue] of the
    /// element.
    variable_roots: VariableRoots,

    // TODO: docs
    constraints: Constraints,
}

impl ConstraintBuilder {
    pub fn new(symbol_table: SymbolTable) -> Self {
        let constraints = Constraints::new(symbol_table.num_trace_segments());
        Self {
            symbol_table,
            constrained_boundaries: BTreeSet::new(),
            variable_roots: VariableRoots::default(),
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
                let lhs = self.constraints.insert_trace_access(
                    &self.symbol_table,
                    &trace_access,
                    domain,
                )?;

                // add its expression to the constraints graph.
                let rhs = self.constraints.insert_expr(
                    &self.symbol_table,
                    constraint.value(),
                    &mut self.variable_roots,
                    domain,
                )?;

                // ensure that the inferred trace segment of the rhs expression can be applied to
                // column against which the boundary constraint is applied.
                // trace segment inference defaults to the lowest segment (the main trace) and is
                // adjusted according to the use of random values and trace columns.
                if lhs.trace_segment() < rhs.trace_segment() {
                    return Err(SemanticError::trace_segment_mismatch(lhs.trace_segment()));
                }

                // merge the two sides of the expression into a constraint.
                self.insert_constraint(lhs, rhs)?
            }
            ast::BoundaryStmt::Variable(variable) => {
                //  TODO: deal with expression at this stage?
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
                let lhs = self.constraints.insert_expr(
                    &self.symbol_table,
                    constraint.lhs(),
                    &mut self.variable_roots,
                    ConstraintDomain::EveryRow,
                )?;

                // add the right hand side expression to the graph.
                let rhs = self.constraints.insert_expr(
                    &self.symbol_table,
                    constraint.rhs(),
                    &mut self.variable_roots,
                    ConstraintDomain::EveryRow,
                )?;

                // merge the two sides of the expression into a constraint.
                self.insert_constraint(lhs, rhs)?
            }
            ast::IntegrityStmt::Variable(variable) => {
                //  TODO: deal with expression at this stage?
                if let VariableType::ListComprehension(list_comprehension) = variable.value() {
                    let vector = unfold_lc(list_comprehension, &self.symbol_table)?;
                    self.symbol_table.insert_integrity_variable(Variable::new(
                        Identifier(variable.name().to_string()),
                        VariableType::Vector(vector),
                    ))?
                } else {
                    self.symbol_table.insert_integrity_variable(variable)?
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
        lhs: ExprDetails,
        rhs: ExprDetails,
    ) -> Result<(), SemanticError> {
        let constraint = self.constraints.merge_equal_exprs(&lhs, &rhs)?;
        let trace_segment = constraint.trace_segment() as usize;

        // the constraint should not be against an undeclared trace segment.
        if self.symbol_table.num_trace_segments() <= trace_segment {
            return Err(SemanticError::InvalidConstraint(
                "Constraint against undeclared trace segment".to_string(),
            ));
        }

        // add the constraint to the constraints
        self.constraints.insert_constraint(
            constraint.root_idx(),
            trace_segment,
            constraint.domain(),
        );

        Ok(())
    }
}
