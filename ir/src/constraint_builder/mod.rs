use std::fmt;

use crate::constraints::{AlgebraicGraph, NodeIndex, Operation};

use super::{
    ast, AccessType, BTreeMap, BTreeSet, ConstantType, ConstantValue, ConstraintDomain,
    ConstraintRoot, Constraints, Declarations, Expression, Identifier, Iterable, ListComprehension,
    ListFoldingType, ListFoldingValueType, MatrixAccess, SemanticError, Symbol, SymbolTable,
    SymbolType, TraceAccess, TraceBindingAccess, TraceBindingAccessSize, TraceSegment,
    ValidateAccess, Value, Variable, VariableType, VectorAccess, CURRENT_ROW,
};

mod boundary_constraints;
use air_script_core::TraceBinding;
pub(crate) use boundary_constraints::ConstrainedBoundary;

mod expression;

mod integrity_constraints;

mod variables;
use variables::get_variable_expr;

// CONSTRAINT BUILDER
// ================================================================================================

#[derive(Default, Debug)]
pub enum ConstraintBuilderContext {
    #[default]
    None,
    EvaluatorFunction(Vec<TraceBinding>),
    BoundaryConstraint(ConstraintDomain),
    IntegrityConstraint,
}

impl fmt::Display for ConstraintBuilderContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConstraintBuilderContext::None => write!(f, "None"),
            ConstraintBuilderContext::EvaluatorFunction(_) => write!(f, "EvaluatorFunction"),
            ConstraintBuilderContext::BoundaryConstraint(_) => write!(f, "BoundaryConstraint"),
            ConstraintBuilderContext::IntegrityConstraint => write!(f, "IntegrityConstraint"),
        }
    }
}

// TODO: docs
#[derive(Default, Debug)]
pub(super) struct ConstraintBuilder {
    // TODO: docs
    symbol_table: SymbolTable,

    // --- CONTEXT VARIABLES ----------------------------------------------------------------------
    context: ConstraintBuilderContext,

    /// A set of all boundaries which have been constrained. This is used to ensure that no more
    /// than one constraint is defined at any given boundary.
    constrained_boundaries: BTreeSet<ConstrainedBoundary>,

    // --- ACCUMULATED CONTEXT DATA ---------------------------------------------------------------
    /// A directed acyclic graph which represents all of the constraints and their subexpressions.
    graph: AlgebraicGraph,

    /// Constraint root node and domain for all boundary constraints against the execution trace,
    /// where boundary constraints are any constraints that apply to either the first or the last
    /// row of the trace.
    boundary_constraints: Vec<(NodeIndex, ConstraintDomain)>,

    /// Constraint root nodes for all integrity constraints against the execution trace, where
    /// integrity constraints are any constraints that apply to every row or every frame.
    integrity_constraints: Vec<NodeIndex>,
}

impl ConstraintBuilder {
    pub fn new(symbol_table: SymbolTable) -> Self {
        let num_trace_segments = symbol_table.num_trace_segments();
        Self {
            symbol_table,

            // context variables
            context: ConstraintBuilderContext::None,
            constrained_boundaries: BTreeSet::new(),

            // accumulated data in the current context
            boundary_constraints: Vec::with_capacity(num_trace_segments),
            integrity_constraints: Vec::with_capacity(num_trace_segments),
            graph: AlgebraicGraph::default(),
        }
    }

    /// TODO: docs
    pub fn into_air(self) -> Result<(Declarations, Constraints), SemanticError> {
        let num_trace_segments = self.symbol_table.num_trace_segments();

        let mut boundary_constraints = vec![Vec::new(); num_trace_segments];
        let mut integrity_constraints = vec![Vec::new(); num_trace_segments];

        // process the boundary constraints
        for (root, default_domain) in self.boundary_constraints.into_iter() {
            // get the trace segment and domain of the constraint
            let (trace_segment, domain) = self.graph.node_details(&root, default_domain)?;
            let trace_segment = usize::from(trace_segment);
            validate_trace_segment(num_trace_segments, trace_segment)?;

            let constraint_root = ConstraintRoot::new(root, domain);
            boundary_constraints[trace_segment].push(constraint_root);
        }

        // process the integrity constraints
        for root in self.integrity_constraints.into_iter() {
            // get the trace segment and domain of the constraint
            // the default domain for integrity constraints is EveryRow
            let (trace_segment, domain) =
                self.graph.node_details(&root, ConstraintDomain::EveryRow)?;
            let trace_segment = usize::from(trace_segment);
            validate_trace_segment(num_trace_segments, trace_segment)?;

            let constraint_root = ConstraintRoot::new(root, domain);
            integrity_constraints[trace_segment].push(constraint_root);
        }

        let declarations = self.symbol_table.into_declarations();
        let constraints = Constraints::new(self.graph, boundary_constraints, integrity_constraints);

        Ok((declarations, constraints))
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// TODO: docs
    pub(super) fn insert_graph_node(&mut self, op: Operation) -> NodeIndex {
        self.graph.insert_node(op)
    }

    // TODO: docs
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

        self.context = ConstraintBuilderContext::IntegrityConstraint;
        for stmt in integrity_stmts.into_iter() {
            self.process_integrity_stmt(stmt)?;
        }
        self.symbol_table.clear_variables();

        Ok(())
    }

    fn insert_constraint(&mut self, lhs: Expression, rhs: Expression) -> Result<(), SemanticError> {
        // add the left hand side expression to the graph.
        let lhs = self.insert_expr(lhs)?;

        // add the right hand side expression to the graph.
        let rhs = self.insert_expr(rhs)?;

        // merge the two sides of the expression into a constraint.
        let root = self.merge_equal_exprs(lhs, rhs);

        match self.context {
            ConstraintBuilderContext::EvaluatorFunction(_)
            | ConstraintBuilderContext::IntegrityConstraint => {
                self.integrity_constraints.push(root)
            }
            ConstraintBuilderContext::BoundaryConstraint(domain) => {
                let (lhs_segment, _) = self.graph.node_details(&lhs, domain)?;
                let (rhs_segment, _) = self.graph.node_details(&rhs, domain)?;
                if lhs_segment < rhs_segment {
                    return Err(SemanticError::boundary_constraint_trace_segment_mismatch(
                        lhs_segment,
                        rhs_segment,
                    ));
                }
                self.boundary_constraints.push((root, domain))
            }
            _ => todo!(),
        }

        Ok(())
    }
}

/// TODO: docs
fn validate_trace_segment(
    num_trace_segments: usize,
    trace_segment: usize,
) -> Result<(), SemanticError> {
    // the constraint should not be against an undeclared trace segment.
    if num_trace_segments <= trace_segment {
        return Err(SemanticError::InvalidConstraint(
            "Constraint against undeclared trace segment".to_string(),
        ));
    }

    Ok(())
}
