use super::{
    ast, AccessType, AlgebraicGraph, BTreeMap, BTreeSet, ConstantType, ConstantValue,
    ConstraintDomain, ConstraintRoot, Constraints, Declarations, Expression, Identifier, Iterable,
    ListComprehension, ListFoldingType, ListFoldingValueType, MatrixAccess, NodeIndex, Operation,
    SemanticError, Symbol, SymbolTable, SymbolType, TraceAccess, TraceBindingAccess,
    TraceBindingAccessSize, TraceSegment, ValidateAccess, Value, Variable, VariableType,
    VectorAccess, CURRENT_ROW,
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

    /// A directed acyclic graph which represents all of the constraints and their subexpressions.
    graph: AlgebraicGraph,

    /// Constraint roots for all boundary constraints against the execution trace, by trace segment,
    /// where boundary constraints are any constraints that apply to either the first or the last
    /// row of the trace.
    boundary_constraints: Vec<Vec<ConstraintRoot>>,

    /// Constraint roots for all integrity constraints against the execution trace, by trace segment,
    /// where integrity constraints are any constraints that apply to every row or every frame.
    integrity_constraints: Vec<Vec<ConstraintRoot>>,
}

impl ConstraintBuilder {
    pub fn new(symbol_table: SymbolTable) -> Self {
        let num_trace_segments = symbol_table.num_trace_segments();
        Self {
            symbol_table,

            // context variables
            constrained_boundaries: BTreeSet::new(),

            // accumulated data in the current context
            boundary_constraints: vec![Vec::new(); num_trace_segments],
            integrity_constraints: vec![Vec::new(); num_trace_segments],
            graph: AlgebraicGraph::default(),
        }
    }

    pub fn into_air(self) -> (Declarations, Constraints) {
        let constraints = Constraints::new(
            self.graph,
            self.boundary_constraints,
            self.integrity_constraints,
        );
        (self.symbol_table.into_declarations(), constraints)
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// TODO: docs
    pub(super) fn insert_graph_node(&mut self, op: Operation) -> NodeIndex {
        self.graph.insert_node(op)
    }

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
        let constraint_root = ConstraintRoot::new(root, domain);
        // add the constraint to the appropriate set of constraints.
        if domain.is_boundary() {
            self.boundary_constraints[trace_segment].push(constraint_root);
        } else {
            self.integrity_constraints[trace_segment].push(constraint_root);
        }

        Ok(())
    }
}
