use air_ir::{Air, TraceSegmentId};
use codegen::Function;

use crate::air::graph::constraint_to_string;

/// Adds the main boundary constraints to the generated code.
pub(super) fn add_main_boundary_constraints(eval_func: &mut Function, ir: &Air) {
    eval_func.line("");
    eval_func.line("// Main boundary constraints");
    for constraint in ir.boundary_constraints(TraceSegmentId::Main) {
        let assertion = constraint_to_string(ir, constraint, true);
        eval_func.line(assertion);
    }
}

#[allow(dead_code)]
/// Adds the aux boundary constraints to the generated code.
pub(super) fn add_aux_boundary_constraints(eval_func: &mut Function, ir: &Air) {
    eval_func.line("");
    eval_func.line("// Aux boundary constraints");
    for constraint in ir.boundary_constraints(TraceSegmentId::Aux) {
        let assertion = constraint_to_string(ir, constraint, true);
        eval_func.line(assertion);

        // TODO: better check assumptions on aux boundary constraints:
        // - start only with empty buses,
        // - end with values derived from builder.aux_bus_boundary_value()
    }
}
