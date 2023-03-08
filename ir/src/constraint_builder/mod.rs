use super::{
    ast, AccessType, BTreeMap, BTreeSet, ConstantType, ConstantValue, ConstraintDomain,
    Constraints, Declarations, Expression, Identifier, IndexedTraceAccess, Iterable,
    ListComprehension, ListFoldingType, ListFoldingValueType, MatrixAccess, NamedTraceAccess,
    NodeIndex, Operation, SemanticError, Symbol, SymbolTable, SymbolType, TraceSegment,
    ValidateAccess, Value, Variable, VariableType, VectorAccess, CURRENT_ROW,
};

mod boundary_constraints;
pub(crate) use boundary_constraints::ConstrainedBoundary;

mod integrity_constraints;

mod expression;

mod variables;
use variables::get_variable_expr;

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

    // TODO: docs
    pub(crate) fn insert_boundary_constraints(
        &mut self,
        stmts: Vec<ast::BoundaryStmt>,
    ) -> Result<(), SemanticError> {
        for stmt in stmts.into_iter() {
            self.insert_boundary_stmt(stmt)?
        }
        self.symbol_table.clear_variables();

        Ok(())
    }

    // TODO: docs
    pub(crate) fn insert_integrity_constraints(
        &mut self,
        stmts: Vec<ast::IntegrityStmt>,
    ) -> Result<(), SemanticError> {
        for stmt in stmts.into_iter() {
            self.insert_integrity_stmt(stmt)?
        }
        self.symbol_table.clear_variables();

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
