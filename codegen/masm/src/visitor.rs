use crate::constants::{AUX_TRACE, MAIN_TRACE};
use air_ir::{
    AccessType, Air, ConstraintDomain, ConstraintRoot, IntegrityConstraintDegree, NodeIndex,
    Operation, PeriodicColumn, PublicInput, TraceAccess, TraceSegmentId, Value,
};

pub trait AirVisitor<'ast> {
    type Value;
    type Error;

    fn visit_access_type(&mut self, access: &'ast AccessType) -> Result<Self::Value, Self::Error>;

    fn visit_boundary_constraint(
        &mut self,
        constraint: &'ast ConstraintRoot,
        trace_segment: TraceSegmentId,
    ) -> Result<Self::Value, Self::Error>;

    fn visit_air(&mut self) -> Result<Self::Value, Self::Error>;

    fn visit_integrity_constraint_degree(
        &mut self,
        constraint: IntegrityConstraintDegree,
        trace_segment: TraceSegmentId,
    ) -> Result<Self::Value, Self::Error>;

    fn visit_integrity_constraint(
        &mut self,
        constraint: &'ast ConstraintRoot,
        trace_segment: TraceSegmentId,
    ) -> Result<Self::Value, Self::Error>;

    fn visit_node_index(&mut self, node_index: &'ast NodeIndex)
        -> Result<Self::Value, Self::Error>;

    fn visit_operation(&mut self, op: &'ast Operation) -> Result<Self::Value, Self::Error>;

    fn visit_periodic_column(
        &mut self,
        columns: &'ast PeriodicColumn,
    ) -> Result<Self::Value, Self::Error>;

    fn visit_public_input(
        &mut self,
        constant: &'ast PublicInput,
    ) -> Result<Self::Value, Self::Error>;

    fn visit_trace_access(
        &mut self,
        trace_access: &'ast TraceAccess,
    ) -> Result<Self::Value, Self::Error>;

    fn visit_value(&mut self, value: &'ast Value) -> Result<Self::Value, Self::Error>;
}

pub fn walk_public_inputs<'ast, V: AirVisitor<'ast>>(
    visitor: &mut V,
    ir: &'ast Air,
) -> Result<(), V::Error> {
    for input in ir.public_inputs() {
        visitor.visit_public_input(input)?;
    }

    Ok(())
}

pub fn walk_integrity_constraint_degrees<'ast, V: AirVisitor<'ast>>(
    visitor: &mut V,
    ir: &'ast Air,
    trace_segment: TraceSegmentId,
) -> Result<(), V::Error> {
    for constraint in ir.integrity_constraint_degrees(trace_segment) {
        visitor.visit_integrity_constraint_degree(constraint, trace_segment)?;
    }

    Ok(())
}

pub fn walk_periodic_columns<'ast, V: AirVisitor<'ast>>(
    visitor: &mut V,
    ir: &'ast Air,
) -> Result<(), V::Error> {
    for column in ir.periodic_columns() {
        visitor.visit_periodic_column(column)?;
    }

    Ok(())
}

/// Walks the IR's boundary constraints.
///
/// The boundary constraints have an implicit natural order. Defined by:
///
/// - Trace segment: The main trace is sorted before the auxiliary trace.
/// - Row: Constraints on the first row are sorted before the last row.
/// - Column: Constraints on lower columns are sorted before higher columns.
///
/// The order above has two functions:
///
/// - It sorts the constraints so that the order of iteration matches the order in which the
/// composition coefficients are defined.
/// - It sorts the constraints so groups with the same divisor are iterated together.
pub fn walk_boundary_constraints_in_natural_order<'ast, V: AirVisitor<'ast>>(
    visitor: &mut V,
    ir: &'ast Air,
) -> Result<(), V::Error> {
    fn domain(boundary: &&ConstraintRoot) -> u8 {
        match boundary.domain() {
            ConstraintDomain::FirstRow => 0,
            ConstraintDomain::LastRow => 1,
            ConstraintDomain::EveryRow => panic!("EveryRow is not supported"),
            ConstraintDomain::EveryFrame(_) => panic!("EveryFrame is not supported"),
        }
    }
    for segment in [MAIN_TRACE, AUX_TRACE] {
        let mut constraints: Vec<&'ast ConstraintRoot> =
            ir.boundary_constraints(segment).iter().collect();

        // TODO: Sort by the column index. Issue #315
        constraints.sort_by_key(domain);

        for boundary in constraints {
            visitor.visit_boundary_constraint(boundary, segment)?;
        }
    }

    Ok(())
}

pub fn walk_integrity_constraints<'ast, V: AirVisitor<'ast>>(
    visitor: &mut V,
    ir: &'ast Air,
    trace_segment: TraceSegmentId,
) -> Result<(), V::Error> {
    for integrity in ir.integrity_constraints(trace_segment) {
        visitor.visit_integrity_constraint(integrity, trace_segment)?;
    }

    Ok(())
}
