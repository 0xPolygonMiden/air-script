use air_ir::{Air, TraceSegmentId};
use codegen::Function;

use super::Codegen;

/// Adds the main transition constraints to the generated code.
pub(super) fn add_main_transition_constraints(eval_func: &mut Function, ir: &Air) {
    // add the main integrity constraints
    for constraint in ir.integrity_constraints(TraceSegmentId::Main) {
        let expr_root = constraint.node_index();

        let expr_root_string = expr_root.to_string(ir, TraceSegmentId::Main);

        let assertion = format!("builder.when_transition().assert_zero::<_>({expr_root_string});");

        eval_func.line(assertion);
    }
}
