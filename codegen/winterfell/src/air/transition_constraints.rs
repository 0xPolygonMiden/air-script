use super::{AirIR, Impl};
use ir::{
    transition_stmts::{AlgebraicGraph, ConstantValue, Operation},
    NodeIndex,
};

// HELPERS TO GENERATE THE WINTERFELL TRANSITION CONSTRAINT METHODS
// ================================================================================================

/// Adds an implementation of the "evaluate_transition" method to the referenced Air implementation
/// based on the data in the provided AirIR.
pub(super) fn add_fn_evaluate_transition(impl_ref: &mut Impl, ir: &AirIR) {
    // define the function.
    let evaluate_transition = impl_ref
        .new_fn("evaluate_transition")
        .arg_ref_self()
        .generic("E: FieldElement<BaseField = Felt>")
        .arg("frame", "&EvaluationFrame<E>")
        .arg("periodic_values", "&[E]")
        .arg("result", "&mut [E]");

    // declare current and next trace row arrays.
    evaluate_transition.line("let current = frame.current();");
    evaluate_transition.line("let next = frame.next();");

    // output the constraints.
    let graph = ir.transition_graph();
    for (idx, constraint) in ir.transition_constraints(0).iter().enumerate() {
        evaluate_transition.line(format!(
            "result[{}] = {};",
            idx,
            constraint.to_string(graph)
        ));
    }
}

/// Adds an implementation of the "evaluate_aux_transition" method to the referenced Air implementation
/// based on the data in the provided AirIR.
pub(super) fn add_fn_evaluate_aux_transition(impl_ref: &mut Impl, ir: &AirIR) {
    // define the function.
    let evaluate_aux_transition = impl_ref
        .new_fn("evaluate_aux_transition")
        .generic("F, E")
        .arg_ref_self()
        .arg("main_frame", "&EvaluationFrame<F>")
        .arg("aux_frame", "&EvaluationFrame<E>")
        .arg("_periodic_values", "&[F]")
        .arg("aux_rand_elements", "&AuxTraceRandElements<E>")
        .arg("result", "&mut [E]")
        .bound("F", "FieldElement<BaseField = Felt>")
        .bound("E", "FieldElement<BaseField = Felt> + ExtensionOf<F>");

    // declare current and next trace row arrays.
    evaluate_aux_transition.line("let current = aux_frame.current();");
    evaluate_aux_transition.line("let next = aux_frame.next();");

    // output the constraints.
    let graph = ir.transition_graph();
    for (idx, constraint) in ir.transition_constraints(1).iter().enumerate() {
        evaluate_aux_transition.line(format!(
            "result[{}] = {};",
            idx,
            constraint.to_string(graph)
        ));
    }
}

/// Code generation trait for generating Rust code strings from [AlgebraicGraph] types.
trait Codegen {
    fn to_string(&self, graph: &AlgebraicGraph) -> String;
}

impl Codegen for NodeIndex {
    fn to_string(&self, graph: &AlgebraicGraph) -> String {
        let op = graph.node(self).op();
        op.to_string(graph)
    }
}

impl Codegen for Operation {
    // TODO: Only add parentheses in Add and Mul if the expression is an arithmetic operation.
    fn to_string(&self, graph: &AlgebraicGraph) -> String {
        match self {
            Operation::Constant(ConstantValue::Inline(value)) => format!("E::from({}_u64)", value),
            Operation::Constant(ConstantValue::Scalar(ident)) => format!("E::from({})", ident),
            Operation::Constant(ConstantValue::Vector(vector_access)) => {
                format!("E::from({}[{}])", vector_access.name(), vector_access.idx())
            }
            Operation::Constant(ConstantValue::Matrix(matrix_access)) => format!(
                "E::from({}[{}][{}])",
                matrix_access.name(),
                matrix_access.row_idx(),
                matrix_access.col_idx()
            ),
            Operation::TraceElement(trace_access) => match trace_access.row_offset() {
                0 => {
                    format!("current[{}]", trace_access.col_idx())
                }
                1 => {
                    format!("next[{}]", trace_access.col_idx())
                }
                _ => panic!("Winterfell doesn't support row offsets greater than 1."),
            },
            Operation::PeriodicColumn(col_idx, _) => {
                format!("periodic_values[{}]", col_idx)
            }
            Operation::RandomValue(idx) => {
                format!("aux_rand_elements.get_segment_elements(0)[{}]", idx)
            }
            Operation::Neg(idx) => {
                let str = idx.to_string(graph);
                format!("- ({})", str)
            }
            Operation::Add(l_idx, r_idx) => {
                let lhs = l_idx.to_string(graph);
                let rhs = r_idx.to_string(graph);

                format!("{} + {}", lhs, rhs)
            }
            Operation::Sub(l_idx, r_idx) => {
                let lhs = l_idx.to_string(graph);
                let rhs = r_idx.to_string(graph);

                format!("{} - ({})", lhs, rhs)
            }
            Operation::Mul(l_idx, r_idx) => {
                let lhs = l_idx.to_string(graph);
                let rhs = r_idx.to_string(graph);
                format!("({}) * ({})", lhs, rhs)
            }
            Operation::Exp(l_idx, r_idx) => {
                let lhs = l_idx.to_string(graph);
                format!("({}).exp(E::PositiveInteger::from({}_u64))", lhs, r_idx)
            }
        }
    }
}
