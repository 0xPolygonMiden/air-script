use super::{AirIR, Impl};
use ir::{
    transition_constraints::{AlgebraicGraph, Operation},
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
        .generic("E: FieldElement<BaseField = Felt>")
        .arg_ref_self()
        .arg("frame", "&EvaluationFrame<E>")
        .arg("periodic_values", "&[E]")
        .arg("result", "&mut [E]");

    // declare current and next trace row arrays.
    evaluate_transition.line("let current = frame.current();");
    evaluate_transition.line("let next = frame.next();");

    // output the constraints.
    let graph = ir.main_transition_graph();
    for (idx, constraint) in ir.main_transition_constraints().iter().enumerate() {
        evaluate_transition.line(format!(
            "result[{}] = {};",
            idx,
            constraint.to_string(graph)
        ));
    }
}

// RUST STRING GENERATION
// ================================================================================================

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
            Operation::Const(value) => format!("E::from({})", value),
            Operation::MainTraceCurrentRow(col_idx) => {
                format!("current[{}]", col_idx)
            }
            Operation::MainTraceNextRow(col_idx) => {
                format!("next[{}]", col_idx)
            }
            Operation::PeriodicColumn(col_idx) => {
                format!("periodic_values[{}]", col_idx)
            }
            Operation::Neg(idx) => {
                let str = idx.to_string(graph);
                format!("- ({})", str)
            }
            Operation::Add(l_idx, r_idx) => {
                let lhs = l_idx.to_string(graph);

                // output Add followed by Neg as "-"
                let rhs = if let Operation::Neg(n_idx) = graph.node(r_idx).op() {
                    format!("- ({})", n_idx.to_string(graph))
                } else {
                    format!("+ {}", r_idx.to_string(graph))
                };
                format!("{} {}", lhs, rhs)
            }
            Operation::Mul(l_idx, r_idx) => {
                let lhs = l_idx.to_string(graph);
                let rhs = r_idx.to_string(graph);
                format!("({}) * ({})", lhs, rhs)
            }
            Operation::Exp(l_idx, r_idx) => {
                let lhs = l_idx.to_string(graph);
                format!("({}).exp({})", lhs, r_idx)
            }
        }
    }
}
