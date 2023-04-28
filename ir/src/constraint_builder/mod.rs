use super::{
    ast, AccessType, AlgebraicGraph, BTreeMap, BTreeSet, ComprehensionContext, ConstantValueExpr,
    ConstraintDomain, ConstraintRoot, Constraints, Declarations, Expression, Identifier, Iterable,
    ListComprehension, ListFolding, ListFoldingValueExpr, NodeIndex, Operation, SemanticError,
    Symbol, SymbolAccess, SymbolBinding, SymbolTable, TraceAccess, TraceSegment, Value,
    VariableBinding, VariableValueExpr, CURRENT_ROW,
};

mod boundary_constraints;
pub(crate) use boundary_constraints::ConstrainedBoundary;

mod evaluators;
pub(crate) use evaluators::Evaluator;

mod integrity_constraints;

mod expression;

mod variables;
use variables::get_variable_expr;

// CONSTRAINT BUILDER
// ================================================================================================

/// A builder that constructs a constraint graph from a symbol table, a set of evaluators, and
/// [BoundaryStmt] and [IntegrityStmt] statements that define variable bindings and constraints.
#[derive(Default, Debug)]
pub(super) struct ConstraintBuilder {
    /// A symbol table that contains all symbols that are visible in the current context.
    symbol_table: SymbolTable,

    // --- CONTEXT VARIABLES ----------------------------------------------------------------------
    /// A set of all boundaries which have been constrained. This is used to ensure that no more
    /// than one constraint is defined at any given boundary.
    constrained_boundaries: BTreeSet<ConstrainedBoundary>,

    /// A map of all evaluator functions that have been defined so far with their names as keys.
    evaluators: BTreeMap<String, Evaluator>,

    // --- ACCUMULATED CONTEXT DATA ---------------------------------------------------------------
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
    /// Initializes a new [ConstraintBuilder] from the specified [SymbolTable] and set of
    /// evaluator functions.
    pub fn new(symbol_table: SymbolTable, evaluators: BTreeMap<String, Evaluator>) -> Self {
        let num_trace_segments = symbol_table.num_trace_segments();
        Self {
            symbol_table,

            // context variables
            constrained_boundaries: BTreeSet::new(),
            evaluators,

            // accumulated data in the current context
            boundary_constraints: vec![Vec::new(); num_trace_segments],
            integrity_constraints: vec![Vec::new(); num_trace_segments],
            graph: AlgebraicGraph::default(),
        }
    }

    /// Consumes this [ConstraintBuilder] and returns a tuple of [Declarations] and [Constraints].
    pub fn into_air(self) -> (Declarations, Constraints) {
        let constraints = Constraints::new(
            self.graph,
            self.boundary_constraints,
            self.integrity_constraints,
        );
        (self.symbol_table.into_declarations(), constraints)
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// Adds the specified operation to the graph and returns the index of its node.
    pub(super) fn insert_graph_node(&mut self, op: Operation) -> NodeIndex {
        self.graph.insert_node(op)
    }

    /// Processes the provided boundary and integrity statements, which consist of variables and
    /// constraint definitions in the boundary and integrity contexts. The graph and the respective
    /// [ConstraintRoot] matrices are updated.
    ///
    /// # Errors
    /// Returns an error if any of the statements are invalid.
    pub(crate) fn insert_constraints(
        &mut self,
        boundary_stmts: Vec<ast::BoundaryStmt>,
        integrity_stmts: Vec<ast::IntegrityStmt>,
    ) -> Result<(), SemanticError> {
        // --- PROCESS BOUNDARY STATEMENTS --------------------------------------------------------

        for stmt in boundary_stmts.into_iter() {
            self.process_boundary_stmt(stmt)?
        }
        self.symbol_table.clear_variables();

        // --- PROCESS INTEGRITY STATEMENTS -------------------------------------------------------

        for stmt in integrity_stmts.into_iter() {
            self.process_integrity_stmt(stmt)?
        }
        self.symbol_table.clear_variables();

        Ok(())
    }

    /// Inserts a [ConstraintRoot] for the constraint specified by the root, trace_segment, and
    /// domain into the correct constraints matrix (boundary_constraints or integrity_constraints).
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
            if self.integrity_constraints.len() <= trace_segment {
                // resize the integrity constraints vector to include the new trace segment
                // this can be required when processing evaluators, since the trace declarations
                // may not have been processed with the [ConstraintBuilder] was initialized.
                self.integrity_constraints
                    .resize(trace_segment + 1, Vec::new());
            }
            self.integrity_constraints[trace_segment].push(constraint_root);
        }

        Ok(())
    }
}
