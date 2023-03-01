use core::panic;

use super::{
    AirIR, AlgebraicGraph, Codegen, ConstraintDomain, ElemType, Impl, IndexedTraceAccess,
    NodeIndex, Operation, Value,
};

// HELPERS TO GENERATE THE WINTERFELL BOUNDARY CONSTRAINT METHODS
// ================================================================================================

/// Adds an implementation of the "get_assertions" method to the referenced Air implementation
/// based on the data in the provided AirIR.
/// TODO: add result types to these functions.
pub(super) fn add_fn_get_assertions(impl_ref: &mut Impl, ir: &AirIR) {
    // define the function
    let get_assertions = impl_ref
        .new_fn("get_assertions")
        .arg_ref_self()
        .ret("Vec<Assertion<Felt>>");

    // add the boundary constraints
    add_assertions(get_assertions, ir, 0);

    // return the result
    get_assertions.line("result");
}

/// Adds an implementation of the "get_aux_assertions" method to the referenced Air implementation
/// based on the data in the provided AirIR.
pub(super) fn add_fn_get_aux_assertions(impl_ref: &mut Impl, ir: &AirIR) {
    // define the function
    let get_aux_assertions = impl_ref
        .new_fn("get_aux_assertions")
        .generic("E: FieldElement<BaseField = Felt>")
        .arg_ref_self()
        .arg("aux_rand_elements", "&AuxTraceRandElements<E>")
        .ret("Vec<Assertion<E>>");

    // add the boundary constraints
    add_assertions(get_aux_assertions, ir, 1);

    // return the result
    get_aux_assertions.line("result");
}

/// Declares a result vector and adds assertions for boundary constraints to it for the specified
/// trace segment
fn add_assertions(func_body: &mut codegen::Function, ir: &AirIR, trace_segment: u8) {
    let elem_type = if trace_segment == 0 {
        ElemType::Base
    } else {
        ElemType::Ext
    };

    // declare the result vector to be returned.
    func_body.line("let mut result = Vec::new();");

    // add the boundary constraints
    for constraint in ir.boundary_constraints(trace_segment) {
        let (trace_access, expr_root) =
            split_boundary_constraint(ir.constraint_graph(), constraint.node_index());
        debug_assert!(trace_access.trace_segment() == trace_segment);

        let assertion = format!(
            "result.push(Assertion::single({}, {}, {}));",
            trace_access.col_idx(),
            domain_to_str(constraint.domain()),
            expr_root.to_string(ir, elem_type, trace_segment)
        );
        func_body.line(assertion);
    }
}

/// Returns a string slice representing the provided constraint domain.
fn domain_to_str(domain: ConstraintDomain) -> String {
    match domain {
        ConstraintDomain::FirstRow => "0".to_string(),
        ConstraintDomain::LastRow => "self.last_step()".to_string(),
        // TODO: replace this with an Error once we have a Result return type.
        _ => panic!("invalid constraint domain"),
    }
}

// CONSTRAINT GRAPH HELPERS
// ================================================================================================

/// Given a node index that is expected to be the root index of a boundary constraint, returns
/// the [IndexedTraceAccess] representing the trace segment and column against which the
/// boundary constraint expression must hold, as well as the node index that represents the root
/// of the constraint expression that must equal zero during evaluation.
///
/// TODO: replace panics with Result and Error
pub fn split_boundary_constraint(
    graph: &AlgebraicGraph,
    index: &NodeIndex,
) -> (IndexedTraceAccess, NodeIndex) {
    let node = graph.node(index);
    match node.op() {
        Operation::Sub(lhs, rhs) => {
            if let Operation::Value(Value::TraceElement(trace_access)) = graph.node(lhs).op() {
                debug_assert!(trace_access.row_offset() == 0);
                (*trace_access, *rhs)
            } else {
                panic!("InvalidUsage: index {index:?} is not the constraint root of a boundary constraint");
            }
        }
        _ => panic!("InvalidUsage: index {index:?} is not the root index of a constraint"),
    }
}
