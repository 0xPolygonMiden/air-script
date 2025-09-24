use air_ir::{Air, TraceSegmentId};
use codegen::Function;

use super::Codegen;

/// Adds the main boundary constraints to the generated code.
pub(super) fn add_main_boundary_constraints(eval_func: &mut Function, ir: &Air) {
    for constraint in ir.boundary_constraints(TraceSegmentId::Main) {
        let expr_root = constraint.node_index();

        let expr_root_string = expr_root.to_string(ir);

        let assertion = match constraint.domain() {
            air_ir::ConstraintDomain::FirstRow => {
                format!("builder.when_first_row().assert_zero::<_>({expr_root_string});")
            },
            air_ir::ConstraintDomain::LastRow => {
                format!("builder.when_last_row().assert_zero::<_>({expr_root_string});")
            },
            _ => unreachable!("Boundary constraints can only be applied to the first or last row"),
        };
        eval_func.line(assertion);
    }
}
