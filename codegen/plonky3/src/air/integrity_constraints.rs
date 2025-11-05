use air_ir::{Air, ConstraintDomain, TraceSegmentId};
use codegen::Function;

use super::Codegen;

/// Adds the main integrity constraints to the generated code.
pub(super) fn add_main_integrity_constraints(eval_func: &mut Function, ir: &Air) {
    for constraint in ir.integrity_constraints(TraceSegmentId::Main) {
        let expr_root = constraint.node_index();
        let expr_root_string = expr_root.to_string(ir);

        // If the constraint is a transition constraint (depends on the next row), we do not
        // evaluate it in the last row, with the `when_transition` method.
        let assertion = if let ConstraintDomain::EveryFrame(_) = constraint.domain() {
            format!("builder.when_transition().assert_zero_ext::<_>({expr_root_string});")
        } else {
            format!("builder.assert_zero_ext::<_>({expr_root_string});")
        };

        eval_func.line(assertion);
    }
}
