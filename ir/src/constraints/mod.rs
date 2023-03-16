use super::{ast::Boundary, SemanticError, TraceSegment, Value};
use std::collections::BTreeMap;

mod constraint;
pub use constraint::{ConstraintDomain, ConstraintRoot};

mod degree;
pub use degree::IntegrityConstraintDegree;

mod graph;
pub use graph::{AlgebraicGraph, NodeIndex, Operation};

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

    /// Constraint roots for all integrity constraints against the execution trace, by trace segment,
    /// where integrity constraints are any constraints that apply to every row or every frame.
    integrity_constraints: Vec<Vec<ConstraintRoot>>,

    /// A directed acyclic graph which represents all of the constraints and their subexpressions.
    graph: AlgebraicGraph,
}

impl Constraints {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    pub fn new(
        graph: AlgebraicGraph,
        boundary_constraints: Vec<Vec<ConstraintRoot>>,
        integrity_constraints: Vec<Vec<ConstraintRoot>>,
    ) -> Self {
        Self {
            graph,
            boundary_constraints,
            integrity_constraints,
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

    /// Returns a vector of the degrees of the integrity constraints for the specified trace
    /// segment.
    pub fn integrity_constraint_degrees(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<IntegrityConstraintDegree> {
        if self.integrity_constraints.len() <= trace_segment.into() {
            return Vec::new();
        }

        self.integrity_constraints[trace_segment as usize]
            .iter()
            .map(|entry_index| self.graph.degree(entry_index.node_index()))
            .collect()
    }

    /// Returns all integrity constraints against the specified trace segment as a vector of
    /// references to [ConstraintRoot] where each index is the tip of the subgraph representing the
    /// constraint within the [AlgebraicGraph].
    pub fn integrity_constraints(&self, trace_segment: TraceSegment) -> &[ConstraintRoot] {
        if self.integrity_constraints.len() <= trace_segment.into() {
            return &[];
        }

        &self.integrity_constraints[trace_segment as usize]
    }

    /// Returns the [AlgebraicGraph] representing all constraints and sub-expressions.
    pub fn graph(&self) -> &AlgebraicGraph {
        &self.graph
    }
}
