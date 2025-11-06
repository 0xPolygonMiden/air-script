use air_ir::{Air, TraceSegmentId};
use codegen::Function;

use crate::air::graph::constraint_to_string;

/// Adds the main boundary constraints to the generated code.
pub(super) fn add_main_boundary_constraints(eval_func: &mut Function, ir: &Air) {
    let constraints = ir.boundary_constraints(TraceSegmentId::Main);
    if !constraints.is_empty() {
        eval_func.line("");
        eval_func.line("// Main boundary constraints");
    }
    for constraint in constraints {
        let assertion = constraint_to_string(ir, constraint, true);
        eval_func.line(assertion);
    }
}

/// Adds the aux boundary constraints to the generated code.
pub(super) fn add_aux_boundary_constraints(eval_func: &mut Function, ir: &Air) {
    let constraints = ir.boundary_constraints(TraceSegmentId::Aux);
    if !constraints.is_empty() {
        eval_func.line("");
        eval_func.line("// Aux boundary constraints");
    }
    for constraint in constraints {
        let assertion = constraint_to_string(ir, constraint, true);
        eval_func.line(assertion);

        // TODO: add code to handle aux boundary constraints

        /*// For first row constraints, we can use `when_first_row` to conditionally apply the assertion
        if constraint.domain() == ConstraintDomain::FirstRow {
            let expr_root = constraint.node_index();
            let expr_root_string = expr_root.to_string(ir, ElemType::Ext);

            let assertion = format!("builder.when_first_row().assert_zero_ext({expr_root_string});");
            eval_func.line(assertion);
        } else {
            let assertion = constraint_to_string(ir, constraint, false);

            let domain_flag = get_domain_flag_str(&constraint.domain());
            let extension_field_flag = get_extension_field_flag_str(ir, *expr_root);
            let assertion = format!("builder{domain_flag}.assert_zero{extension_field_flag}({expr_root_string});");

            eval_func.line(assertion);
        }*/
    }
}
