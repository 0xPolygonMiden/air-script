use air_ir::{Air, TraceSegmentId};
use codegen::Function;

use crate::air::graph::constraint_to_string;

/// Adds the main integrity constraints to the generated code.
pub(super) fn add_main_integrity_constraints(eval_func: &mut Function, ir: &Air) {
    eval_func.line("");
    eval_func.line("// Main integrity/transition constraints");
    for constraint in ir.integrity_constraints(TraceSegmentId::Main) {
        let assertion = constraint_to_string(ir, constraint, false);
        eval_func.line(assertion);
    }
}

/// Adds the aux integrity constraints to the generated code.
pub(super) fn add_aux_integrity_constraints(eval_func: &mut Function, ir: &Air) {
    eval_func.line("");
    eval_func.line("// Aux integrity/transition constraints");
    for constraint in ir.integrity_constraints(TraceSegmentId::Aux) {
        let assertion = constraint_to_string(ir, constraint, false);
        eval_func.line(assertion);
    }
}
