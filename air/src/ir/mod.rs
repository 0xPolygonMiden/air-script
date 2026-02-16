mod bus;
mod constraints;
mod degree;
mod operation;
mod random_inputs;
mod trace;
mod value;

pub use air_parser::{
    Symbol,
    ast::{
        AccessType, Boundary, Identifier, PeriodicColumn, PublicInput, QualifiedIdentifier,
        TraceSegmentId,
    },
};

pub use self::{
    bus::{Bus, BusBoundary, BusOp, BusOpKind, BusType, PublicInputTableAccess},
    constraints::{ConstraintDomain, ConstraintError, ConstraintRoot, Constraints},
    degree::IntegrityConstraintDegree,
    operation::Operation,
    random_inputs::RandomInputs,
    trace::TraceAccess,
    value::{PeriodicColumnAccess, PublicInputAccess, Value},
};

/// A fixed two segment trace shape containing values for the main and aux segments.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TraceShape<T> {
    pub main: T,
    pub aux: T,
}

impl<T> TraceShape<T> {
    pub fn new(main: T, aux: T) -> Self {
        Self { main, aux }
    }

    pub fn map<U, F: FnMut(&T) -> U>(&self, mut f: F) -> TraceShape<U> {
        TraceShape { main: f(&self.main), aux: f(&self.aux) }
    }

    /// Returns an iterator over mutable references to `(TraceSegmentId, T)` in segment order.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (TraceSegmentId, &mut T)> {
        let (main, aux) = (&mut self.main, &mut self.aux);
        [(TraceSegmentId::Main, main), (TraceSegmentId::Aux, aux)].into_iter()
    }
}

impl<T> core::ops::Index<TraceSegmentId> for TraceShape<T> {
    type Output = T;
    fn index(&self, index: TraceSegmentId) -> &Self::Output {
        match index {
            TraceSegmentId::Main => &self.main,
            TraceSegmentId::Aux => &self.aux,
        }
    }
}

impl<T> core::ops::IndexMut<TraceSegmentId> for TraceShape<T> {
    fn index_mut(&mut self, index: TraceSegmentId) -> &mut Self::Output {
        match index {
            TraceSegmentId::Main => &mut self.main,
            TraceSegmentId::Aux => &mut self.aux,
        }
    }
}

impl<T> core::ops::Index<usize> for TraceShape<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.main,
            1 => &self.aux,
            _ => panic!("invalid segment index"),
        }
    }
}

impl<T> core::ops::IndexMut<usize> for TraceShape<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.main,
            1 => &mut self.aux,
            _ => panic!("invalid segment index"),
        }
    }
}

/// A fixed three segment trace shape containing values for the main, aux, and quotient segments.
///
/// This wraps a two segment `TraceShape<T>` for the witness traces, and adds a separate
/// `quotient` segment which is not addressable via `TraceSegmentId`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct FullTraceShape<T> {
    pub segments: TraceShape<T>,
    pub quotient: T,
}

impl<T> FullTraceShape<T> {
    pub fn new(main: T, aux: T, quotient: T) -> Self {
        Self {
            segments: TraceShape::new(main, aux),
            quotient,
        }
    }

    #[inline]
    pub fn segments(&self) -> &TraceShape<T> {
        &self.segments
    }

    #[inline]
    pub fn segments_mut(&mut self) -> &mut TraceShape<T> {
        &mut self.segments
    }
}

impl<T> core::ops::Index<TraceSegmentId> for FullTraceShape<T> {
    type Output = T;
    fn index(&self, index: TraceSegmentId) -> &Self::Output {
        &self.segments[index]
    }
}

impl<T> core::ops::IndexMut<TraceSegmentId> for FullTraceShape<T> {
    fn index_mut(&mut self, index: TraceSegmentId) -> &mut Self::Output {
        &mut self.segments[index]
    }
}

impl<T> core::ops::Index<usize> for FullTraceShape<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.segments.main,
            1 => &self.segments.aux,
            2 => &self.quotient,
            _ => panic!("invalid segment index"),
        }
    }
}

impl<T> core::ops::IndexMut<usize> for FullTraceShape<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.segments.main,
            1 => &mut self.segments.aux,
            2 => &mut self.quotient,
            _ => panic!("invalid segment index"),
        }
    }
}

/// The offset of the "current" row during constraint evaluation.
pub const CURRENT_ROW: usize = 0;
/// The minimum cycle length of a periodic column
pub const MIN_CYCLE_LENGTH: usize = 2;

extern crate alloc;
use alloc::collections::BTreeMap;

use miden_diagnostics::{SourceSpan, Spanned};

use crate::graph::AlgebraicGraph;

/// The intermediate representation of a complete AirScript program
///
/// This structure is produced from an [air_parser::ast::Program] that has
/// been through semantic analysis, constant propagation, and inlining. It
/// is equivalent to an [air_parser::ast::Program], except that it has been
/// translated into an algebraic graph representation, on which further analysis,
/// optimization, and code generation are performed.
#[derive(Debug, Spanned)]
pub struct Air {
    /// The name of the [air_parser::ast::Program] from which this IR was derived
    #[span]
    pub name: Identifier,
    /// The widths (number of columns) of each segment of the trace, in segment order (i.e. the
    /// index in this vector matches the index of the segment in the program).
    pub trace_segment_widths: Vec<u16>,
    /// The periodic columns referenced by this program.
    ///
    /// These are taken straight from the [air_parser::ast::Program] without modification.
    pub periodic_columns: BTreeMap<QualifiedIdentifier, PeriodicColumn>,
    /// The public inputs referenced by this program.
    ///
    /// These are taken straight from the [air_parser::ast::Program] without modification.
    pub public_inputs: BTreeMap<Identifier, PublicInput>,
    /// The total number of elements in the random values array
    pub num_random_values: u16,
    /// Expected maximum constraint id (inclusive) when tag validation is enabled.
    pub expected_max_constraint_id: Option<u64>,
    /// The constraints enforced by this program, in their algebraic graph representation.
    pub constraints: Constraints,
    /// The buses referenced by this program.
    ///
    /// Only their name, type, and the first and last boundary constraints are stored here.
    pub buses: BTreeMap<Identifier, Bus>,
}
impl Default for Air {
    fn default() -> Self {
        Self::new(Identifier::new(SourceSpan::UNKNOWN, Symbol::intern("unnamed")))
    }
}
impl Air {
    /// Create a new, empty [Air] container
    ///
    /// An empty [Air] is meaningless until it has been populated with
    /// constraints and associated metadata. This is typically done by converting
    /// an [air_parser::ast::Program] to this struct using the [crate::passes::MirToAir]
    /// translation pass.
    pub fn new(name: Identifier) -> Self {
        Self {
            name,
            trace_segment_widths: vec![],
            periodic_columns: Default::default(),
            public_inputs: Default::default(),
            num_random_values: 0,
            expected_max_constraint_id: None,
            constraints: Default::default(),
            buses: Default::default(),
        }
    }

    /// Returns the name of the [air_parser::ast::Program] this [Air] was derived from, as a `str`
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn public_inputs(&self) -> impl Iterator<Item = &PublicInput> + '_ {
        self.public_inputs.values()
    }

    /// Returns a list of all accesses to reduced public input tables in canonical order.
    pub fn reduced_public_input_table_accesses(&self) -> Vec<PublicInputTableAccess> {
        let mut accesses: Vec<_> = self
            .buses
            .values()
            .flat_map(|bus| {
                [bus.first, bus.last]
                    .iter()
                    .filter_map(|boundary| {
                        if let BusBoundary::PublicInputTable(access) = boundary {
                            Some(*access)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        accesses.sort();
        accesses.dedup();
        accesses
    }

    pub fn periodic_columns(&self) -> impl Iterator<Item = &PeriodicColumn> + '_ {
        self.periodic_columns.values()
    }

    /// Return the number of boundary constraints
    pub fn num_boundary_constraints(&self, trace_segment: TraceSegmentId) -> usize {
        self.constraints.num_boundary_constraints(trace_segment)
    }

    /// Return the set of [ConstraintRoot] corresponding to the boundary constraints
    pub fn boundary_constraints(&self, trace_segment: TraceSegmentId) -> &[ConstraintRoot] {
        self.constraints.boundary_constraints(trace_segment)
    }

    /// Return the set of [ConstraintRoot] corresponding to the integrity constraints
    pub fn integrity_constraints(&self, trace_segment: TraceSegmentId) -> &[ConstraintRoot] {
        self.constraints.integrity_constraints(trace_segment)
    }

    /// Return the set of [IntegrityConstraintDegree] corresponding to each integrity constraint
    pub fn integrity_constraint_degrees(
        &self,
        trace_segment: TraceSegmentId,
    ) -> Vec<IntegrityConstraintDegree> {
        self.constraints.integrity_constraint_degrees(trace_segment)
    }

    /// Return an [Iterator] over the validity constraints for the given trace segment
    pub fn validity_constraints(
        &self,
        trace_segment: TraceSegmentId,
    ) -> impl Iterator<Item = &ConstraintRoot> + '_ {
        self.constraints
            .integrity_constraints(trace_segment)
            .iter()
            .filter(|constraint| matches!(constraint.domain(), ConstraintDomain::EveryRow))
    }

    /// Return an [Iterator] over the transition constraints for the given trace segment
    pub fn transition_constraints(
        &self,
        trace_segment: TraceSegmentId,
    ) -> impl Iterator<Item = &ConstraintRoot> + '_ {
        self.constraints
            .integrity_constraints(trace_segment)
            .iter()
            .filter(|constraint| matches!(constraint.domain(), ConstraintDomain::EveryFrame(_)))
    }

    /// Return a reference to the raw [AlgebraicGraph] corresponding to the constraints
    #[inline]
    pub fn constraint_graph(&self) -> &AlgebraicGraph {
        self.constraints.graph()
    }

    /// Return a mutable reference to the raw [AlgebraicGraph] corresponding to the constraints
    #[inline]
    pub fn constraint_graph_mut(&mut self) -> &mut AlgebraicGraph {
        self.constraints.graph_mut()
    }
}
