use crate::utils::contraint_root_domain;
use ir::constraints::ConstraintDomain;
use ir::{
    constraints::{ConstraintRoot, Operation},
    AccessType, AirIR, ConstantBinding, IntegrityConstraintDegree, NodeIndex, PeriodicColumn,
    PublicInput, TraceAccess, Value,
};

pub trait AirVisitor<'ast> {
    type Value;
    type Error;

    fn visit_access_type(&mut self, access: &'ast AccessType) -> Result<Self::Value, Self::Error>;

    fn visit_boundary_constraint(
        &mut self,
        constraint: &'ast ConstraintRoot,
        trace_segment: u8,
    ) -> Result<Self::Value, Self::Error>;

    fn visit_air(&mut self) -> Result<Self::Value, Self::Error>;

    fn visit_constant_binding(
        &mut self,
        constant: &'ast ConstantBinding,
    ) -> Result<Self::Value, Self::Error>;

    fn visit_integrity_constraint_degree(
        &mut self,
        constraint: IntegrityConstraintDegree,
        trace_segment: u8,
    ) -> Result<Self::Value, Self::Error>;

    fn visit_integrity_constraint(
        &mut self,
        constraint: &'ast ConstraintRoot,
        trace_segment: u8,
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

pub fn walk_constant_bindings<'ast, V: AirVisitor<'ast>>(
    visitor: &mut V,
    ir: &'ast AirIR,
) -> Result<(), V::Error> {
    for constant in ir.constants() {
        visitor.visit_constant_binding(constant)?;
    }

    Ok(())
}

pub fn walk_public_inputs<'ast, V: AirVisitor<'ast>>(
    visitor: &mut V,
    ir: &'ast AirIR,
) -> Result<(), V::Error> {
    for input in ir.public_inputs() {
        visitor.visit_public_input(input)?;
    }

    Ok(())
}

pub fn walk_integrity_constraint_degrees<'ast, V: AirVisitor<'ast>>(
    visitor: &mut V,
    ir: &'ast AirIR,
    trace_segment: u8,
) -> Result<(), V::Error> {
    for constraint in ir.integrity_constraint_degrees(trace_segment) {
        visitor.visit_integrity_constraint_degree(constraint, trace_segment)?;
    }

    Ok(())
}

pub fn walk_periodic_columns<'ast, V: AirVisitor<'ast>>(
    visitor: &mut V,
    ir: &'ast AirIR,
) -> Result<(), V::Error> {
    for column in ir.periodic_columns() {
        visitor.visit_periodic_column(column)?;
    }

    Ok(())
}

pub fn walk_boundary_constraints<'ast, V: AirVisitor<'ast>>(
    visitor: &mut V,
    ir: &'ast AirIR,
    segment: u8,
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
    ir: &'ast AirIR,
    trace_segment: u8,
) -> Result<(), V::Error> {
    for integrity in ir.integrity_constraints(trace_segment) {
        visitor.visit_integrity_constraint(integrity, trace_segment)?;
    }

    Ok(())
}
