use crate::symbol_table::TraceParameterAccess;

use super::{AlgebraicGraph, NodeIndex, SemanticError, TraceAccess, TraceBinding, Value};
use std::collections::BTreeMap;

// EVALUATORS
// ================================================================================================

/// TODO
#[derive(Default, Debug, Clone)]
pub(crate) struct Evaluator {
    /// TODO: docs
    params: Vec<TraceBinding>,

    /// A vector of the node indices in this graph which contain parameter references.
    param_nodes: BTreeMap<TraceParameterAccess, NodeIndex>,

    /// A list of root indices for each constraint defined in this evaluator function.
    constraints: Vec<NodeIndex>,

    /// A directed acyclic graph which represents all of the constraints defined in this evaluator
    /// and their subexpressions.
    graph: AlgebraicGraph,
}

impl Evaluator {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    pub fn new(
        params: Vec<TraceBinding>,
        param_nodes: BTreeMap<TraceParameterAccess, NodeIndex>,
        constraints: Vec<NodeIndex>,
        graph: AlgebraicGraph,
    ) -> Self {
        Self {
            params,
            param_nodes,
            constraints,
            graph,
        }
    }

    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    /// Returns a graph of the evaluator function call.
    pub fn apply(
        &self,
        args: Vec<TraceAccess>,
    ) -> Result<(AlgebraicGraph, Vec<NodeIndex>), SemanticError> {
        // validate args against evaluator params
        let mut arg_mapping = BTreeMap::new();
        for (param, arg) in self.params.iter().zip(args.iter()) {
            arg_mapping.insert(param.name(), arg);
            // Don't think this should be an error since the arg could be a vector access or
            // if param.size() != arg.size() {
            //     todo!("Error")
            // } else {
            //     arg_mapping.insert(param.name(), arg);
            // }
        }

        let mut graph = self.graph.clone();
        for (_, idx) in self.param_nodes.iter() {
            let node = graph.node(idx);
            let value = node.value();
            match value {
                Value::Parameter(param_access) => {
                    let trace_ref = arg_mapping.get(param_access.name()).ok_or_else(|| {
                        SemanticError::undeclared_parameter(param_access.name().to_owned())
                    })?;
                    debug_assert!(param_access.trace_segment() == trace_ref.trace_segment());
                    let trace_access = TraceAccess::new(
                        param_access.trace_segment(),
                        trace_ref.col_idx() + param_access.idx(),
                        1,
                        param_access.row_offset(),
                    );
                    graph.replace_value_node(*idx, Value::TraceElement(trace_access));
                }
                _ => {
                    todo!("Error")
                }
            }
        }

        Ok((graph, self.constraints.clone()))
    }
}
