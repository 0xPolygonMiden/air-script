use super::{
    ast::{EvaluatorFunction, EvaluatorFunctionCall},
    AlgebraicGraph, BTreeMap, ConstraintBuilder, ConstraintRoot, SemanticError, SymbolAccess,
};

impl ConstraintBuilder {
    /// Builds a new [Evaluator] from the provided [EvaluatorFunction] declaration, adds it to the
    /// existing list of evaluators, and returns all processed evaluators.
    pub(crate) fn process_evaluator(
        mut self,
        ev_decl: EvaluatorFunction,
    ) -> Result<BTreeMap<String, Evaluator>, SemanticError> {
        let (name, trace_params, integrity_stmts) = ev_decl.into_parts();

        // insert all of the parameters as trace bindings
        self.symbol_table.insert_trace_bindings(trace_params)?;

        // process all of the integrity variables and costraints
        for stmt in integrity_stmts {
            self.process_integrity_stmt(stmt)?;
        }

        let mut evaluators = self.evaluators;
        let evaluator = Evaluator::new(self.graph, self.integrity_constraints);
        evaluators.insert(name, evaluator);

        Ok(evaluators)
    }

    /// Looks up the evaluator being called. If it exists, the evaluator's graph and constraints are
    /// duplicated and updated according to the arguments specified by the [EvaluatorFunctionCall],
    /// then added to the existing graph and integrity constraints of this [ConstraintBuilder].
    pub(super) fn process_evaluator_call(
        &mut self,
        ev_call: EvaluatorFunctionCall,
    ) -> Result<(), SemanticError> {
        let (name, args) = ev_call.into_parts();

        // get the evaluator from the list
        if let Some(evaluator) = self.evaluators.get(&name) {
            // get the offsets for the trace binding accesses as a vector of indices
            let trace_offsets = self.get_evaluator_trace_offsets(&args)?;

            // get the index by which nodes from the evaluator graph will be offset
            let node_idx_offset = self.graph.num_nodes();

            // clone the evaluator function's graph and constraint roots and update trace access
            // indices and graph node indices according to the specified offsets
            let (ev_call_graph, mut constraints) =
                evaluator.clone_with_offsets(&trace_offsets, node_idx_offset);

            // extend the constraint graph with all of the nodes from the evaluator graph
            self.graph.extend(ev_call_graph);

            // add the roots of the evaluator function call's constraints
            self.integrity_constraints.append(&mut constraints);
        } else {
            return Err(SemanticError::evaluator_fn_not_declared(&name));
        }

        Ok(())
    }

    // --- HELPER FUNCTIONS -----------------------------------------------------------------------

    /// Maps each [SymbolAccess] in an evaluator function call's list of arguments to one or
    /// more offset values that indicate where in the trace
    ///
    /// It returns a vector for each trace segment in the arguments, where each vector contains as
    /// many offset values as the width of the segment.
    fn get_evaluator_trace_offsets(
        &self,
        args: &[Vec<SymbolAccess>],
    ) -> Result<Vec<Vec<usize>>, SemanticError> {
        // get the offsets for the trace binding accesses as a vector of indices
        let mut offsets: Vec<Vec<usize>> = Vec::new();
        for segment in args.iter() {
            let mut segment_offsets = Vec::new();
            for trace_binding_access in segment {
                let trace_access = self.symbol_table.get_trace_access(trace_binding_access)?;

                // bindings referencing more than one trace element must be split into one offset
                // per trace element.
                for inner_offset in 0..trace_access.size() {
                    segment_offsets.push(trace_access.col_idx() + inner_offset);
                }
            }
            offsets.push(segment_offsets);
        }

        Ok(offsets)
    }
}

// EVALUATOR FUNCTION STRUCT
// ================================================================================================

/// Contains an [AlgebraicGraph] and set of [ConstraintRoot] for an evaluator function definition.
/// The column indices of each [TraceAccess] in the graph are indexed from zero, according to the
/// [TraceBinding] parameters specified by the evaluator function definition.
///
/// When using the evaluator in an [EvaluatorFunctionCall], the column index in each [TraceAccess]
/// must be reindexed according to the column offsets of the arguments specified by the call.
#[derive(Default, Debug, Clone)]
pub(crate) struct Evaluator {
    /// The [ConstraintRoot]s for each constraint defined in this evaluator function, grouped by
    /// the [TraceSegment] to which they apply.
    constraints: Vec<Vec<ConstraintRoot>>,

    /// A directed acyclic graph which represents all of the constraints defined in this evaluator
    /// and their subexpressions.
    graph: AlgebraicGraph,
}

impl Evaluator {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    /// Creates a new [Evaluator] from the provided [AlgebraicGraph] and [ConstraintRoot] matrix.
    pub(super) fn new(graph: AlgebraicGraph, constraints: Vec<Vec<ConstraintRoot>>) -> Self {
        Self { constraints, graph }
    }

    /// Clones the [AlgebraicGraph] and constraints of this [Evaluator] and updates them according
    /// to the provided offset values.
    /// - trace_offsets is used to update the column indices of the nodes containing [TraceAccess]
    ///   values that reference the execution trace.
    /// - node_idx_offset is used to update the [NodeIndex] of each of the evaluator's
    ///   [ConstraintRoot]s.
    pub(super) fn clone_with_offsets(
        &self,
        trace_offsets: &[Vec<usize>],
        node_idx_offset: usize,
    ) -> (AlgebraicGraph, Vec<Vec<ConstraintRoot>>) {
        // clone the graph, updating the nodes which reference the trace by the offsets.
        let ev_call_graph = self.graph.clone_with_offsets(trace_offsets);

        // create a new set of [ConstraintRoot] with updated [NodeIndex] for each constraint
        let constraints = self
            .constraints
            .iter()
            .map(|segment| {
                // re-index each node in the segment by the node index offset.
                segment
                    .iter()
                    .map(|root| {
                        ConstraintRoot::new(
                            root.node_index().clone_with_offset(node_idx_offset),
                            root.domain(),
                        )
                    })
                    .collect()
            })
            .collect::<Vec<_>>();

        (ev_call_graph, constraints)
    }
}
