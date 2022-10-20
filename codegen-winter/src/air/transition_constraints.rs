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
    fn to_string(&self, graph: &AlgebraicGraph) -> String {
        let mut strings = vec![];

        match self {
            Operation::Constant(value) => strings.push(format!("E::from({})", value)),
            Operation::MainTraceCurrentRow(col_idx) => {
                strings.push(format!("current[{}]", col_idx));
            }
            Operation::MainTraceNextRow(col_idx) => {
                strings.push(format!("next[{}]", col_idx));
            }
            Operation::Neg(idx) => {
                strings.push(String::from("-"));
                let str = idx.to_string(graph);
                strings.push(str);
            }
            Operation::Add(l_idx, r_idx) => {
                let lhs = l_idx.to_string(graph);
                strings.push(lhs);

                // output Add followed by Neg as "-"
                let r_idx = if let Operation::Neg(n_idx) = graph.node(r_idx).op() {
                    strings.push(String::from("-"));
                    n_idx
                } else {
                    strings.push(String::from("+"));
                    r_idx
                };

                let rhs = r_idx.to_string(graph);

                strings.push(rhs);
            }
            _ => {
                // TODO: Mul, Exp
            }
        }

        strings.join("")
    }
}
