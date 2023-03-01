use super::{
    ast::Boundary, AccessType, ConstantType, ExprDetails, Expression, Identifier,
    IndexedTraceAccess, ListFoldingType, MatrixAccess, SemanticError, SymbolAccess, SymbolTable,
    SymbolType, TraceSegment, VariableRoots, VariableType, VectorAccess,
};
use std::collections::BTreeMap;

mod constraint;
pub use constraint::{ConstrainedBoundary, ConstraintDomain, ConstraintRoot};

mod degree;
pub use degree::IntegrityConstraintDegree;

mod graph;
pub use graph::{AlgebraicGraph, ConstantValue, NodeIndex, Operation};

// CONSTANTS
// ================================================================================================

/// The default segment against which a constraint is applied is the main trace segment.
const DEFAULT_SEGMENT: TraceSegment = 0;
/// The auxiliary trace segment.
const AUX_SEGMENT: TraceSegment = 1;
/// The offset of the "current" row during constraint evaluation.
pub(super) const CURRENT_ROW: usize = 0;
/// TODO: docs
pub(super) const MIN_CYCLE_LENGTH: usize = 2;

// CONSTRAINTS
// ================================================================================================

/// Contains the graph representing all of the constraints and their subexpressions, the set of
/// variables used in the integrity constraints, and a matrix for each constraint type (boundary,
/// validity, transition), where each index contains a vector of the constraint roots for all the
/// constraints of that type against the segment of the trace corresponding to that index. For
/// example, transition constraints against the main execution trace, which is trace segment 0, will
/// be specified by a vector in transition_constraints[0] containing a [ConstraintRoot] in the graph
/// for each constraint against the main trace.
#[derive(Default, Debug)]
pub(crate) struct Constraints {
    /// Constraint roots for all boundary constraints against the execution trace, by trace segment,
    /// where boundary constraints are any constraints that apply to either the first or the last
    /// row of the trace.
    boundary_constraints: Vec<Vec<ConstraintRoot>>,

    /// Constraint roots for all validity constraints against the execution trace, by trace segment,
    /// where validity constraints are any constraints that apply to every row.
    validity_constraints: Vec<Vec<ConstraintRoot>>,

    /// Constraint roots for all transition constraints against the execution trace, by trace
    /// segment, where transition constraints are any constraints that apply to a frame of multiple
    /// rows.
    transition_constraints: Vec<Vec<ConstraintRoot>>,

    /// A directed acyclic graph which represents all of the constraints and their subexpressions.
    graph: AlgebraicGraph,
}

impl Constraints {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    /// TODO: these constraint vectors should be initialized to the proper length
    pub fn new(num_trace_segments: usize) -> Self {
        Self {
            boundary_constraints: vec![Vec::new(); num_trace_segments],
            validity_constraints: vec![Vec::new(); num_trace_segments],
            transition_constraints: vec![Vec::new(); num_trace_segments],
            graph: AlgebraicGraph::default(),
        }
    }

    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    /// Returns the number of boundary constraints applied against the specified trace segment.
    pub fn num_boundary_constraints(&self, trace_segment: TraceSegment) -> usize {
        if self.boundary_constraints.len() <= trace_segment.into() {
            return 0;
        }

        self.boundary_constraints[trace_segment as usize].len()
    }

    /// Returns all boundary constraints against the specified trace segment as a slice of
    /// [ConstraintRoot] where each index is the tip of the subgraph representing the constraint
    /// within the constraints [AlgebraicGraph].
    pub fn boundary_constraints(&self, trace_segment: TraceSegment) -> &[ConstraintRoot] {
        if self.boundary_constraints.len() <= trace_segment.into() {
            return &[];
        }

        &self.boundary_constraints[trace_segment as usize]
    }

    /// Returns a vector of the degrees of the validity constraints for the specified trace
    /// segment.
    pub fn validity_constraint_degrees(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<IntegrityConstraintDegree> {
        if self.validity_constraints.len() <= trace_segment.into() {
            return Vec::new();
        }

        self.validity_constraints[trace_segment as usize]
            .iter()
            .map(|entry_index| self.graph.degree(entry_index.node_index()))
            .collect()
    }

    /// Returns all validity constraints against the specified trace segment as a vector of
    /// references to [ConstraintRoot] where each index is the tip of the subgraph representing the
    /// constraint within the [AlgebraicGraph].
    pub fn validity_constraints(&self, trace_segment: TraceSegment) -> &[ConstraintRoot] {
        if self.validity_constraints.len() <= trace_segment.into() {
            return &[];
        }

        &self.validity_constraints[trace_segment as usize]
    }

    /// Returns a vector of the degrees of the transition constraints for the specified trace
    /// segment.
    pub fn transition_constraint_degrees(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<IntegrityConstraintDegree> {
        if self.transition_constraints.len() <= trace_segment.into() {
            return Vec::new();
        }

        self.transition_constraints[trace_segment as usize]
            .iter()
            .map(|entry_index| self.graph.degree(entry_index.node_index()))
            .collect()
    }

    /// Returns all transition constraints against the specified trace segment as a vector of
    /// references to [ConstraintRoot] where each index is the tip of the subgraph representing the
    /// constraint within the [AlgebraicGraph].
    pub fn transition_constraints(&self, trace_segment: TraceSegment) -> &[ConstraintRoot] {
        if self.transition_constraints.len() <= trace_segment.into() {
            return &[];
        }

        &self.transition_constraints[trace_segment as usize]
    }

    /// Returns the [AlgebraicGraph] representing all constraints and sub-expressions.
    pub fn graph(&self) -> &AlgebraicGraph {
        &self.graph
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    // TODO: get rid of this
    pub(super) fn insert_expr(
        &mut self,
        symbol_table: &SymbolTable,
        expr: &Expression,
        variable_roots: &mut VariableRoots,
        default_domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        self.graph
            .insert_expr(symbol_table, expr, variable_roots, default_domain)
    }

    // TODO: get rid of this
    pub(super) fn insert_trace_access(
        &mut self,
        symbol_table: &SymbolTable,
        trace_access: &IndexedTraceAccess,
        domain: ConstraintDomain,
    ) -> Result<ExprDetails, SemanticError> {
        self.graph
            .insert_trace_access(symbol_table, trace_access, domain)
    }

    // TODO: get rid of this
    pub(super) fn merge_equal_exprs(
        &mut self,
        lhs: &ExprDetails,
        rhs: &ExprDetails,
    ) -> Result<ExprDetails, SemanticError> {
        self.graph.merge_equal_exprs(lhs, rhs)
    }

    pub(super) fn insert_constraint(
        &mut self,
        node_idx: NodeIndex,
        trace_segment: usize,
        domain: ConstraintDomain,
    ) {
        let constraint_root = ConstraintRoot::new(node_idx, domain);

        // add the constraint to the appropriate set of constraints.
        match domain {
            ConstraintDomain::FirstRow | ConstraintDomain::LastRow => {
                self.boundary_constraints[trace_segment].push(constraint_root);
            }
            ConstraintDomain::EveryRow => {
                self.validity_constraints[trace_segment].push(constraint_root);
            }
            ConstraintDomain::EveryFrame(_) => {
                self.transition_constraints[trace_segment].push(constraint_root);
            }
        }
    }
}
