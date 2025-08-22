extern crate alloc;
use alloc::collections::BTreeSet;
use core::fmt;

use super::*;
use crate::graph::{AlgebraicGraph, NodeIndex};

#[derive(Debug, thiserror::Error)]
pub enum ConstraintError {
    #[error("cannot merge incompatible constraint domains ({0} and {1})")]
    IncompatibleConstraintDomains(ConstraintDomain, ConstraintDomain),
}

/// [Constraints] is the algebraic graph representation of all the constraints in an AirScript. The
/// graph contains all of the constraints, each of which is a subgraph consisting of all the
/// expressions involved in evaluating the constraint, including constants, references to the trace,
/// public inputs, random values, and periodic columns.
///
/// Internally, this struct also holds a matrix for each constraint type (boundary, integrity),
/// where each row corresponds to a trace segment (in the same order) and contains a vector of
/// [ConstraintRoot] for all of the constraints of that type to be applied to that trace segment.
///
/// For example, integrity constraints for the main execution trace, which has a trace segment id of
/// 0, will be specified by the vector of constraint roots found at index 0 of the
/// `integrity_constraints` matrix.
#[derive(Default, Debug)]
pub struct Constraints {
    /// Constraint roots for all boundary constraints against the execution trace, by trace
    /// segment, where boundary constraints are any constraints that apply to either the first
    /// or the last row of the trace.
    boundary_constraints: BTreeMap<TraceSegmentId, Vec<ConstraintRoot>>,
    /// Constraint roots for all integrity constraints against the execution trace, by trace
    /// segment, where integrity constraints are any constraints that apply to every row or
    /// every frame.
    integrity_constraints: BTreeMap<TraceSegmentId, Vec<ConstraintRoot>>,
    /// A directed acyclic graph which represents all of the constraints and their subexpressions.
    graph: AlgebraicGraph,
}
impl Constraints {
    /// Constructs a new [Constraints] graph from the given parts
    pub const fn new(
        graph: AlgebraicGraph,
        boundary_constraints: BTreeMap<TraceSegmentId, Vec<ConstraintRoot>>,
        integrity_constraints: BTreeMap<TraceSegmentId, Vec<ConstraintRoot>>,
    ) -> Self {
        Self {
            graph,
            boundary_constraints,
            integrity_constraints,
        }
    }

    /// Updates the root boundary and integrity constraints to use the new node indices  
    /// values, given in the `renumbering_map`.  
    ///  
    /// This functions also removes duplicate constraints (that share the same root and domain).  
    ///  
    /// # Panics  
    /// Panics if a constraint's node index is not found in the renumbering map.
    pub fn renumber_and_deduplicate_constraints(
        &mut self,
        renumbering_map: &BTreeMap<NodeIndex, NodeIndex>,
    ) {
        // Iterate over all boundary and integrity constraints
        for (_, segment_constraints) in self
            .boundary_constraints
            .iter_mut()
            .chain(self.integrity_constraints.iter_mut())
        {
            let mut added_indices = BTreeSet::new();
            segment_constraints.retain_mut(|constraint| {
                let new_index = *renumbering_map
                    .get(constraint.node_index())
                    .expect("Error: cannot find constraint index in renumbering map");
                // Don't keep duplicate constraints
                if !added_indices.insert((new_index, constraint.domain)) {
                    return false;
                }
                // If this constraint is new, we update its node index and keep it
                constraint.update_node_index(new_index);
                true
            });
        }
    }

    /// Returns the number of boundary constraints applied against the specified trace segment.
    pub fn num_boundary_constraints(&self, trace_segment: TraceSegmentId) -> usize {
        self.boundary_constraints.get(&trace_segment).map_or(0, |v| v.len())
    }

    /// Returns the set of boundary constraints for the given trace segment.
    ///
    /// Each boundary constraint is represented by a [ConstraintRoot] which is
    /// the root of the subgraph representing the constraint within the [AlgebraicGraph]
    pub fn boundary_constraints(&self, trace_segment: TraceSegmentId) -> &[ConstraintRoot] {
        self.boundary_constraints.get(&trace_segment).map_or(&[], |v| v.as_slice())
    }

    /// Returns a vector of the degrees of the integrity constraints for the specified trace
    /// segment.
    pub fn integrity_constraint_degrees(
        &self,
        trace_segment: TraceSegmentId,
    ) -> Vec<IntegrityConstraintDegree> {
        self.integrity_constraints.get(&trace_segment).map_or(vec![], |v| {
            v.iter()
                .map(|entry_index| self.graph.degree(entry_index.node_index()))
                .collect()
        })
    }

    /// Returns the set of integrity constraints for the given trace segment.
    ///
    /// Each integrity constraint is represented by a [ConstraintRoot] which is
    /// the root of the subgraph representing the constraint within the [AlgebraicGraph]
    pub fn integrity_constraints(&self, trace_segment: TraceSegmentId) -> &[ConstraintRoot] {
        self.integrity_constraints.get(&trace_segment).map_or(&[], |v| v.as_slice())
    }

    /// Inserts a new constraint against `trace_segment`, using the provided `root` and `domain`
    pub fn insert_constraint(
        &mut self,
        trace_segment: TraceSegmentId,
        root: NodeIndex,
        domain: ConstraintDomain,
    ) {
        let root = ConstraintRoot::new(root, domain);
        if domain.is_boundary() {
            self.boundary_constraints.entry(trace_segment).or_default().push(root);
        } else {
            self.integrity_constraints.entry(trace_segment).or_default().push(root);
        }
    }

    /// Returns the underlying [AlgebraicGraph] representing all constraints and their
    /// sub-expressions.
    #[inline]
    pub const fn graph(&self) -> &AlgebraicGraph {
        &self.graph
    }

    /// Returns a mutable reference to the underlying [AlgebraicGraph] representing all constraints
    /// and their sub-expressions.
    #[inline]
    pub fn graph_mut(&mut self) -> &mut AlgebraicGraph {
        &mut self.graph
    }
}

/// A [ConstraintRoot] represents the entry node of a subgraph within the [AlgebraicGraph]
/// representing a constraint. It also contains the [ConstraintDomain] for the constraint, which is
/// the domain against which the constraint should be applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintRoot {
    index: NodeIndex,
    domain: ConstraintDomain,
}
impl ConstraintRoot {
    /// Creates a new [ConstraintRoot] with the specified entry index and row offset.
    pub const fn new(index: NodeIndex, domain: ConstraintDomain) -> Self {
        Self { index, domain }
    }

    /// Returns the index of the entry node of the subgraph representing the constraint.
    pub const fn node_index(&self) -> &NodeIndex {
        &self.index
    }

    /// Updates the node index this constraint refers to. This should be called if the graph is
    /// updated after its initial construction, such as during common subexpression elimination.
    pub fn update_node_index(&mut self, new_index: NodeIndex) {
        self.index = new_index;
    }

    /// Returns the [ConstraintDomain] for this constraint, which specifies the rows against which
    /// the constraint should be applied.
    pub const fn domain(&self) -> ConstraintDomain {
        self.domain
    }
}

/// [ConstraintDomain] corresponds to the domain over which a constraint is applied.
///
/// See the docs on each variant for more details.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConstraintDomain {
    /// For boundary constraints which apply to the first row
    FirstRow,
    /// For boundary constraints which apply to the last row
    LastRow,
    /// For constraints which apply to every row of the trace
    ///
    /// This is used for validity constraints
    EveryRow,
    /// For constraints which apply across multiple rows at once.
    ///
    /// A "frame" is a window over rows in the trace, i.e. a constraint
    /// over a frame of size 2 is a constraint that observes 2 rows at
    /// a time, at each step of the trace, e.g. current and next rows.
    /// Such a constraint verifies that certain properties hold in the
    /// transition between every pair of rows.
    ///
    /// This is used for transition constraints.
    EveryFrame(usize),
}
impl ConstraintDomain {
    /// Returns true if this domain is a boundary domain (e.g. first or last)
    pub fn is_boundary(&self) -> bool {
        matches!(self, Self::FirstRow | Self::LastRow)
    }

    /// Returns true if this domain is an integrity constraint domain.
    pub fn is_integrity(&self) -> bool {
        matches!(self, Self::EveryRow | Self::EveryFrame(_))
    }

    /// Returns a [ConstraintDomain] corresponding to the given row offset.
    ///
    /// * `offset == 0` corresponds to every row
    /// * `offset > 0` corresponds to a frame size of `offset + 1`
    pub fn from_offset(offset: usize) -> Self {
        if offset == 0 {
            Self::EveryRow
        } else {
            Self::EveryFrame(offset + 1)
        }
    }

    /// Combines two compatible [ConstraintDomain]s into a single [ConstraintDomain]
    /// that represents the maximum of the two.
    ///
    /// For example, if one domain is [ConstraintDomain::EveryFrame(2)] and the other
    /// is [ConstraintDomain::EveryFrame(3)], then the result will be
    /// [ConstraintDomain::EveryFrame(3)].
    ///
    /// NOTE: Domains for boundary constraints (FirstRow and LastRow) cannot be merged with other
    /// domains.
    pub fn merge(self, other: Self) -> Result<Self, ConstraintError> {
        if self == other {
            return Ok(other);
        }

        match (self, other) {
            (Self::EveryFrame(a), Self::EveryRow) => Ok(Self::EveryFrame(a)),
            (Self::EveryRow, Self::EveryFrame(b)) => Ok(Self::EveryFrame(b)),
            (Self::EveryFrame(a), Self::EveryFrame(b)) => Ok(Self::EveryFrame(a.max(b))),
            _ => Err(ConstraintError::IncompatibleConstraintDomains(self, other)),
        }
    }
}
impl From<Boundary> for ConstraintDomain {
    fn from(boundary: Boundary) -> Self {
        match boundary {
            Boundary::First => Self::FirstRow,
            Boundary::Last => Self::LastRow,
        }
    }
}
impl fmt::Display for ConstraintDomain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FirstRow => write!(f, "the first row"),
            Self::LastRow => write!(f, "the last row"),
            Self::EveryRow => write!(f, "every row"),
            Self::EveryFrame(size) => {
                write!(f, "every frame of {size} consecutive rows")
            },
        }
    }
}
