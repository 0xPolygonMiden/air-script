use crate::utils::contraint_root_domain;
use air_ir::{
    Air, ConstraintDomain, ConstraintRoot, NodeIndex, Operation, PeriodicColumn, TraceSegmentId,
    Value,
};

pub trait AirVisitor<'ast> {
    type Value;
    type Error;

    fn visit_boundary_constraint(
        &mut self,
        constraint: &'ast ConstraintRoot,
        trace_segment: TraceSegmentId,
    ) -> Result<Self::Value, Self::Error>;

    fn visit_air(&mut self) -> Result<Self::Value, Self::Error>;

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

    fn visit_value(&mut self, value: &'ast Value) -> Result<Self::Value, Self::Error>;
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

pub fn walk_boundary_constraints<'ast, V: AirVisitor<'ast>>(
    visitor: &mut V,
    ir: &'ast Air,
    segment: TraceSegmentId,
    constraint_domain: ConstraintDomain,
) -> Result<(), V::Error> {
    let mut constraints: Vec<&'ast ConstraintRoot> =
        ir.boundary_constraints(segment).iter().collect();

    constraints.sort_by_key(contraint_root_domain);

    for boundary in constraints
        .iter()
        .filter(|c| c.domain() == constraint_domain)
    {
        visitor.visit_boundary_constraint(boundary, segment)?;
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
