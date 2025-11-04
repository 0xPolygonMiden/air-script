use std::{collections::HashMap, ops::Deref};

use air_pass::Pass;
use miden_diagnostics::{DiagnosticsHandler, Severity, SourceSpan, Spanned};

use super::{duplicate_node_or_replace, visitor::Visitor};
use crate::{
    CompileError,
    ir::{
        Graph, Link, Mir, MirAccessType, MirType, MirValue, Node, Op, Parameter, Parent,
        Root, SpannedMirValue, TraceAccessBinding, Value, Vector,
    },
    passes::constant_propagation::get_inner_const,
};

/// This pass handles inlining of `Call` nodes at their call sites.
///
/// It works in three steps:
/// * Firstly, we visit the graph to build the call dependency graph.
/// * This dependency graph is then used to compute the wanted inlining order (we first replace
///   calls to callees that do not have `Call` in their body). If it is not possible to create this
///   order, this means there is a circular dependency.
/// * Then, we visit the graph again at each `Call` nodes, building a duplicate of the body (with
///   Parameter replaced by call arguments), and replacing the `Call` node by this duplicate body.
///  
pub struct Inlining<'a> {
    diagnostics: &'a DiagnosticsHandler,
}

impl<'a> Inlining<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }

    /// Runs the Inlining pass once (with both InliningFirstPass and InliningSecondPass)
    ///
    /// Returns true if any calls were inlined, false otherwise, to let the caller know if any
    /// changes were made or if we reached a fixed point.
    fn run_once(&mut self, ir: &mut Mir) -> Result<bool, CompileError> {
        // The first pass only identifies the call graph dependencies and the needed calls to inline
        let mut first_pass = InliningFirstPass::new(self.diagnostics);
        Visitor::run(&mut first_pass, ir.constraint_graph_mut())?;

        // We then create the inlining order (inlining first the functions and evaluators that do
        // not call other functions or evaluators)
        let func_eval_inlining_order =
            create_inlining_order(self.diagnostics, first_pass.func_eval_dependency_graph.clone())?;

        // The second pass actually inlines the calls
        let mut second_pass = InliningSecondPass::new(
            self.diagnostics,
            func_eval_inlining_order.clone(),
            first_pass.func_eval_nodes_where_called.clone(),
        );
        Visitor::run(&mut second_pass, ir.constraint_graph_mut())?;

        Ok(second_pass.had_calls)
    }
}

// If we have to run the inlining algorithm `INLINING_LIMIT`, we return an error
const INLINING_LIMIT: usize = 10;

impl Pass for Inlining<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut had_calls = true;
        let mut iterations = 0;

        while had_calls && iterations < INLINING_LIMIT {
            had_calls = self.run_once(&mut ir)?;
            iterations += 1;
        }

        if had_calls {
            self.diagnostics.error(
                "Inlining call depth limit reached, some calls may not have been inlined. Aborting.".to_string(),
            );
            return Err(CompileError::Failed);
        }

        Ok(ir)
    }
}

pub struct InliningFirstPass<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,
    in_func_or_eval: bool,
    // When encountering a call, we store it here to construct the dependency graph once reaching
    // the root node
    current_callees_encountered: Vec<Link<Root>>,
    // HashMap<FunctionPtr, (Function, Functions where it is called)>
    func_eval_dependency_graph: HashMap<usize, (Link<Root>, Vec<Link<Root>>)>,
    // HashMap<CaleePtr, Callee, Vec<Call nodes where called>>
    func_eval_nodes_where_called: HashMap<usize, (Link<Root>, Vec<Link<Op>>)>, // Op is a Call here
}

impl<'a> InliningFirstPass<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self {
            diagnostics,
            work_stack: vec![],
            in_func_or_eval: false,
            current_callees_encountered: Vec::new(),
            func_eval_dependency_graph: HashMap::new(),
            func_eval_nodes_where_called: HashMap::new(),
        }
    }
}

/// This structure is used to keep track of what is needed to inline a call to a given function or
/// evaluator.
#[derive(Clone, Debug)]
pub struct CallInliningContext {
    body: Link<Vec<Link<Op>>>,
    arguments: Link<Vec<Link<Op>>>,
    pure_function: bool,
    ref_node: Link<Node>,
}

pub struct InliningSecondPass<'a> {
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,
    // context for both passes
    func_eval_inlining_order: Vec<Link<Root>>,
    // context for second pass
    call_inlining_context: Option<CallInliningContext>,
    // HashMap<KeyPtr, (Key, Value)>
    nodes_to_replace: HashMap<usize, (Link<Op>, Link<Op>)>,
    params_for_ref_node: HashMap<usize, Vec<Link<Op>>>,

    // HashMap<CaleePtr, (Callee, Vec<Call nodes where called>)>
    func_eval_nodes_where_called: HashMap<usize, (Link<Root>, Vec<Link<Op>>)>, // Op is a Call here
    had_calls: bool,
}

impl<'a> InliningSecondPass<'a> {
    pub fn new(
        diagnostics: &'a DiagnosticsHandler,
        func_eval_inlining_order: Vec<Link<Root>>,
        func_eval_nodes_where_called: HashMap<usize, (Link<Root>, Vec<Link<Op>>)>,
    ) -> Self {
        Self {
            diagnostics,
            work_stack: vec![],
            call_inlining_context: None,
            nodes_to_replace: HashMap::new(),
            params_for_ref_node: HashMap::new(),
            func_eval_nodes_where_called,
            func_eval_inlining_order,
            had_calls: false,
        }
    }
}

/// Helper function to create the inlining order depending on the dependency graph
///
/// Raises an error if a circular dependency is detected
fn create_inlining_order(
    diagnostics: &DiagnosticsHandler,
    mut func_eval_dependency_graph: HashMap<usize, (Link<Root>, Vec<Link<Root>>)>,
) -> Result<Vec<Link<Root>>, CompileError> {
    let mut func_eval_inlining_order = Vec::new();

    // Note: we remove an element at each iteration (or raise diag), so this will terminate
    while !func_eval_dependency_graph.is_empty() {
        // Find a function without dependency
        match func_eval_dependency_graph.clone().iter().find(|(_, (_k, v))| v.is_empty()) {
            Some((f_ptr, (f, _))) => {
                func_eval_inlining_order.push(f.clone());
                // Remove the entry of dependency graph corresponding to the next function to inline
                func_eval_dependency_graph.remove(f_ptr);
            },
            _ => {
                diagnostics
                    .diagnostic(Severity::Error)
                    .with_message("Circular dependency detected")
                    .emit();
                return Err(CompileError::Failed);
            },
        }

        let removed_fn = func_eval_inlining_order.last().unwrap();

        // Remove the function from the list of dependencies of all other functions
        func_eval_dependency_graph.iter_mut().for_each(|(_, (_, v))| {
            v.retain(|x| x != removed_fn);
        });
    }
    Ok(func_eval_inlining_order)
}

impl Visitor for InliningFirstPass<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn run(&mut self, graph: &mut Graph) -> Result<(), CompileError> {
        for root_node in self.root_nodes_to_visit(graph) {
            if let Some(root) = root_node.as_root() {
                if let Some(_function) = root.clone().as_function() {
                    self.in_func_or_eval = true;
                } else if let Some(_evaluator) = root.clone().as_evaluator() {
                    self.in_func_or_eval = true;
                } else {
                    unreachable!("Encountered a root node that is not a Function or an Evaluator");
                }
            } else {
                self.in_func_or_eval = false;
            }

            self.scan_node(graph, root_node.clone())?;
            while let Some(node) = self.work_stack().pop() {
                self.visit_node(graph, node)?;
            }
        }
        Ok(())
    }
    fn root_nodes_to_visit(&self, graph: &Graph) -> Vec<Link<Node>> {
        let functions = graph.get_function_nodes();
        let evaluators = graph.get_evaluator_nodes();
        let boundary_constraints_roots_ref = graph.boundary_constraints_roots.borrow();
        let integrity_constraints_roots_ref = graph.integrity_constraints_roots.borrow();
        let bus_roots: Vec<_> = graph
            .buses
            .values()
            .flat_map(|b| b.borrow().clone().columns.into_iter().collect::<Vec<_>>())
            .collect();

        let combined_roots = boundary_constraints_roots_ref
            .clone()
            .into_iter()
            .map(|bc| bc.as_node())
            .chain(integrity_constraints_roots_ref.clone().into_iter().map(|ic| ic.as_node()))
            .chain(bus_roots.into_iter().map(|b| b.as_node()))
            .chain(evaluators.into_iter().map(|e| e.as_node()))
            .chain(functions.into_iter().map(|f| f.as_node()));
        combined_roots.collect()
    }

    // When visiting a function or an evaluator, we have just finished visiting their bodies so we
    // can update the dependency graph for this root We then clear the
    // current_callees_encountered vec to prepare to visit the next function or evaluator
    fn visit_function(
        &mut self,
        _graph: &mut Graph,
        function: Link<Root>,
    ) -> Result<(), CompileError> {
        self.func_eval_dependency_graph
            .insert(function.get_ptr(), (function, self.current_callees_encountered.clone()));
        self.current_callees_encountered.clear();
        Ok(())
    }
    fn visit_evaluator(
        &mut self,
        _graph: &mut Graph,
        evaluator: Link<Root>,
    ) -> Result<(), CompileError> {
        self.func_eval_dependency_graph.insert(
            evaluator.get_ptr(),
            (evaluator.clone(), self.current_callees_encountered.clone()),
        );
        self.current_callees_encountered.clear();
        Ok(())
    }

    // When visiting a call, we:
    // - add it to the current list of callees if we're currently visiting the bodies of functions
    //   or evaluators (to build the dependency graph)
    // - add it to the list of calls to inline with the func_eval_nodes_where_called map
    fn visit_call(&mut self, _graph: &mut Graph, call: Link<Op>) -> Result<(), CompileError> {
        // safe to unwrap because we just dispatched on it
        let callee = &call.as_call().unwrap().function;

        if self.in_func_or_eval {
            self.current_callees_encountered.push(callee.clone());
        }
        self.func_eval_nodes_where_called
            .entry(callee.get_ptr())
            .and_modify(|(_, v)| v.push(call.clone()))
            .or_insert((callee.clone(), vec![call.clone()]));

        Ok(())
    }
}

impl Visitor for InliningSecondPass<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }

    // Root nodes correspond to all call nodes encountered during the first pass
    // We visit them depending on the inlining order computed based on the dependency graph
    fn root_nodes_to_visit(&self, _graph: &Graph) -> Vec<Link<Node>> {
        let mut call_nodes_to_inline_in_order = Vec::new();
        for callee in self.func_eval_inlining_order.iter() {
            if let Some((_, nodes_with_context)) =
                self.func_eval_nodes_where_called.get(&callee.get_ptr())
            {
                call_nodes_to_inline_in_order
                    .extend(nodes_with_context.iter().map(|call| call.clone().as_node()));
            }
        }
        call_nodes_to_inline_in_order
    }
    fn run(&mut self, graph: &mut Graph) -> Result<(), CompileError> {
        let root_nodes_to_visit = self.root_nodes_to_visit(graph);
        self.had_calls = !root_nodes_to_visit.is_empty();

        for root_node in root_nodes_to_visit {
            let mut updated_op = None;

            if let Some(op) = root_node.as_op() {
                // Set context for inlining this call
                let Some(call_node) = op.as_call() else {
                    return Ok(());
                };

                let callee = call_node.function.clone();
                let arguments = call_node.arguments.clone();
                let (pure_function, body) = if let Some(f) = callee.clone().as_function() {
                    (true, f.body.clone())
                } else if let Some(ev) = callee.clone().as_evaluator() {
                    (false, ev.body.clone())
                } else {
                    unreachable!(
                        "InliningSecondPass::run: callee is not a Function or an Evaluator node"
                    );
                };

                let context = CallInliningContext {
                    body,
                    arguments,
                    pure_function,
                    ref_node: callee.as_node(),
                };

                self.call_inlining_context = Some(context.clone());
                self.nodes_to_replace.clear();
                self.params_for_ref_node.clear();

                self.scan_node(graph, root_node.clone())?;

                while let Some(node) = self.work_stack().pop() {
                    self.visit_node(graph, node.clone())?;
                }

                if context.pure_function {
                    // We have finished inlining the body, we can now replace the Call node with the
                    // last expression of the body
                    let last_child_of_body = context.body.borrow().last().unwrap().clone();
                    let (_, new_node) =
                        self.nodes_to_replace.get(&last_child_of_body.get_ptr()).unwrap().clone();
                    updated_op = Some(new_node);
                } else {
                    // We have finished inlining the body, we can now replace the Call node with all
                    // the body
                    let mut new_nodes = Vec::new();
                    for body_node in context.body.borrow().iter() {
                        // FIXME: Maybe we should only push nodes that are Enf()?
                        // Depends if additional nodes change things (e.g. the Vector size..)
                        // For now I think we can keep all nodes, and just ignore the non-Enf nodes
                        // When building the constraints during lowering Mir -> Air

                        let new_node =
                            self.nodes_to_replace.get(&body_node.get_ptr()).unwrap().1.clone();

                        if let Some(bus_op) = new_node.clone().as_bus_op() {
                            let latch = bus_op.latch.clone();

                            bus_op
                                .bus
                                .to_link()
                                .unwrap()
                                .borrow_mut()
                                .columns
                                .push(new_node.clone());
                            bus_op.bus.to_link().unwrap().borrow_mut().latches.push(latch.clone());
                        }

                        if new_node.clone().as_enf().is_some() {
                            new_nodes.push(new_node);
                        }
                    }
                    let span =
                        new_nodes.iter().map(|n| n.span()).fold(SourceSpan::UNKNOWN, |acc, s| {
                            acc.merge(s).unwrap_or(SourceSpan::UNKNOWN)
                        });
                    let new_nodes_vector = Vector::create(new_nodes, span);

                    updated_op = Some(new_nodes_vector);
                }

                // Reset context to None
                self.call_inlining_context = None;
            }

            // Effectively replace the `Call` node with the updated op
            // Note: We also update the references of Parameters that referenced the node we are
            // replacing
            if let Some(updated_op) = updated_op {
                //
                let prev_owner_ptr = updated_op.as_owner().unwrap().get_ptr();
                let params = self.params_for_ref_node.get(&prev_owner_ptr).cloned();

                root_node.as_op().unwrap().set(&updated_op);

                if let Some(params) = params {
                    let new_owner = root_node.clone().as_op().unwrap().clone().as_owner().unwrap();
                    for param in params.iter() {
                        param.as_parameter_mut().unwrap().set_ref_node(new_owner.clone());
                    }
                }
            }
        }
        Ok(())
    }

    fn scan_node(&mut self, _graph: &Graph, node: Link<Node>) -> Result<(), CompileError> {
        self.work_stack().push(node.clone());
        if let Some(op) = node.clone().as_op() {
            // If we scan a `Call` node, we do not visit its children (the call's arguments)
            // TODO INLINING: Check whether this is the wanted behavior
            if op.as_call().is_some() {
                return Ok(());
            };
            for child in node.children().borrow().iter() {
                self.scan_node(_graph, child.clone().as_node())?;
            }
        }
        Ok(())
    }

    fn visit_call(&mut self, _graph: &mut Graph, _call: Link<Op>) -> Result<(), CompileError> {
        let context = self
            .call_inlining_context
            .clone()
            .expect("InliningSecondPass::visit_node: call_inlining_context is None");
        if context.pure_function {
            // Instead of scanning all the body, we only scan the last node,
            // which represents the return value of the function
            self.scan_node(_graph, context.body.borrow().last().unwrap().clone().as_node())?;
        } else {
            // We scan all the nodes related to the body
            for body_node in context.body.borrow().iter() {
                self.scan_node(_graph, body_node.clone().as_node())?;
            }
        }
        Ok(())
    }

    fn visit_node(&mut self, graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        if node.is_stale() {
            return Ok(());
        }

        {
            let call_op = node.clone().as_op().unwrap_or_else(|| {
                panic!("InliningSecondPass::visit_node on a non-Op node: {node:?}")
            });

            // First, check if it's a known `Call` to inline,
            // if so, set the context and scan its body
            if call_op.clone().as_call().is_some() {
                self.visit_call(graph, call_op.clone())?;
            } else {
                // Else, we are currently visiting the body of a function or an evaluator of a call
                // we want to inline We use our helper duplicate_node_or_replace to
                // duplicate the body, while replacing the `Function` or `Evaluator` parameters with
                // the `Call` arguments
                if self.call_inlining_context.clone().unwrap().pure_function {
                    duplicate_node_or_replace(
                        &mut self.nodes_to_replace,
                        call_op.clone(),
                        self.call_inlining_context.clone().unwrap().arguments.borrow().clone(),
                        self.call_inlining_context.clone().unwrap().ref_node,
                        None,
                        &mut self.params_for_ref_node,
                    );
                } else {
                    // If we're inside the body of an evaluator, we first need to unpack the
                    // arguments of the call to have a `Vector` of `TraceColumn`, and not
                    // bindings to multiple columns
                    let args =
                        self.call_inlining_context.clone().unwrap().arguments.borrow().clone();

                    let callee_params = self
                        .call_inlining_context
                        .clone()
                        .unwrap()
                        .ref_node
                        .as_root()
                        .unwrap()
                        .as_evaluator()
                        .unwrap()
                        .parameters
                        .clone();

                    check_evaluator_argument_sizes(&args, callee_params, self.diagnostics)?;

                    let args_unpacked = unpack_evaluator_arguments(&args);

                    duplicate_node_or_replace(
                        &mut self.nodes_to_replace,
                        call_op.clone(),
                        args_unpacked,
                        self.call_inlining_context.clone().unwrap().ref_node,
                        None,
                        &mut self.params_for_ref_node,
                    );
                }
            }
        }

        Ok(())
    }
}

/// Helper function to recursively extract the outermost accessors
/// if the op is an accessor, otherwise return the op unchanged
fn extract_accessor(op: Link<Op>) -> Link<Op> {
    let Some(accessor) = op.as_accessor() else {
        return op;
    };
    let mut indexable = accessor.indexable.clone();
    match &accessor.access_type {
        MirAccessType::Default => accessor.indexable.clone(),
        MirAccessType::Index(idx_op) => {
            let idx = get_inner_const(idx_op)
                .unwrap_or_else(|| panic!("expected constant index, got {:#?}", idx_op))
                as usize;
            while indexable.clone().as_accessor().is_some() {
                indexable = extract_accessor(indexable.clone());
            }
            match indexable.borrow().deref() {
                Op::Vector(v) => v.children().borrow()[idx].clone(),
                Op::Matrix(m) => m.children().borrow()[idx].clone(),
                _ => unreachable!("expected vector or matrix, got {:#?}", indexable),
            }
        },
        MirAccessType::Matrix(row, col) => {
            let row = get_inner_const(row)
                .unwrap_or_else(|| panic!("expected constant row, got {:#?}", row))
                as usize;
            let col = get_inner_const(col)
                .unwrap_or_else(|| panic!("expected constant column, got {:#?}", col))
                as usize;
            while indexable.clone().as_accessor().is_some() {
                indexable = extract_accessor(indexable.clone());
            }
            match indexable.borrow().deref() {
                Op::Matrix(m) => {
                    m.children().borrow()[row].clone().as_matrix().unwrap().children().borrow()[col]
                        .clone()
                },
                _ => unreachable!("expected matrix, got {:#?}", indexable),
            }
        },
    }
}

/// Helper function to check, for each trace segment, that the total size of arguments is correct
fn check_evaluator_argument_sizes(
    args: &[Link<Op>],
    callee_params: Vec<Vec<Link<Op>>>,
    diagnostics: &DiagnosticsHandler,
) -> Result<(), CompileError> {
    for ((trace_segment_id, trace_segments_params), trace_segments_arg) in
        callee_params.iter().enumerate().zip(args.iter())
    {
        let trace_segments_arg_vector = trace_segments_arg
            .as_vector()
            .unwrap_or_else(|| panic!("expected vector, got {trace_segments_arg:?}"));
        let children = trace_segments_arg_vector.children();
        let (trace_segments_arg_vector_len, _) = process_evaluator_arg_children(children.clone());
        if trace_segments_params.len() != trace_segments_arg_vector_len {
            diagnostics
                .diagnostic(Severity::Error)
                .with_message("argument count mismatch")
                .with_primary_label(
                    SourceSpan::UNKNOWN,
                    format!(
                        "expected call to have {} arguments in trace segment {}, but got {}",
                        trace_segments_params.len(),
                        trace_segment_id,
                        trace_segments_arg_vector_len
                    ),
                )
                .with_secondary_label(
                    SourceSpan::UNKNOWN,
                    format!(
                        "this functions has {} parameters in trace segment {}",
                        trace_segments_params.len(),
                        trace_segment_id
                    ),
                )
                .emit();
            return Err(CompileError::Failed);
        }
    }
    Ok(())
}

/// Helper function to unpack the arguments of a call to an evaluator
fn unpack_evaluator_arguments(args: &[Link<Op>]) -> Vec<Link<Op>> {
    let mut args_unpacked = Vec::new();
    for args_for_trace_segment in args.iter() {
        let trace_segment_vec = args_for_trace_segment.as_vector().expect(
            "Arguments of a Call node to Evaluator should be a Vector for each trace segment",
        );
        let children = trace_segment_vec.children();
        let (_, unpacked_children) = process_evaluator_arg_children(children.clone());
        args_unpacked.extend(unpacked_children);
    }
    args_unpacked
}

fn process_evaluator_arg_children(children: Link<Vec<Link<Op>>>) -> (usize, Vec<Link<Op>>) {
    let mut trace_segments_arg_vector_len = 0;
    let mut args_unpacked = Vec::new();
    for child in children.borrow().deref() {
        // We need to extract the outermost accessors, if any.
        // This can happen when slices are used to rebind
        // trace bindings in nested evaluator calls
        let child = extract_accessor(child.clone());
        if let Some(value) = child.as_value() {
            let Value {
                value: SpannedMirValue { value, span }, ..
            } = value.deref();

            let param_size = match value {
                MirValue::TraceAccessBinding(tab) => {
                    if tab.size > 1 {
                        for index in 0..tab.size {
                            let new_arg = Value::create(SpannedMirValue {
                                value: MirValue::TraceAccessBinding(TraceAccessBinding {
                                    size: 1,
                                    segment: tab.segment,
                                    offset: tab.offset + index,
                                }),
                                span: *span,
                            });
                            args_unpacked.push(new_arg);
                        }
                    } else {
                        args_unpacked.push(child.clone());
                    }
                    tab.size
                },
                MirValue::TraceAccess(_ta) => {
                    args_unpacked.push(child.clone());
                    1
                },
                _ => unreachable!("expected trace access binding or trace access, got {:?}", value),
            };
            trace_segments_arg_vector_len += param_size;
        } else if let Some(parameter) = child.as_parameter() {
            args_unpacked.push(child.clone());
            let Parameter { ty, .. } = parameter.deref();
            let size = match ty {
                MirType::Felt => 1,
                MirType::Vector(len) => *len,
                _ => unreachable!("expected felt or vector, got {:?}", ty),
            };
            trace_segments_arg_vector_len += size;
        } else if let Some(vector) = child.as_vector() {
            // When slices are used to rebind trace bindings in nested evaluator calls,
            // translation from Ast to Mir inserts a Vector of
            // Accessor(MirAccessType::Index), representing the unwrapped
            // slice.
            // We need to unpack all the elements of this vector and add them
            // to the arguments
            // We also need to make sure that this vector is a vector of Felts
            // and to add its length to the total length of arguments
            for c in vector.children().borrow().iter() {
                let c = extract_accessor(c.clone());
                if let Some(param) = c.as_parameter() {
                    assert!(
                        param.ty == MirType::Felt,
                        "expected parameter of type Felt, got {:#?}",
                        param
                    );
                    args_unpacked.push(c.clone());
                } else if let Some(value) = c.as_value() {
                    let SpannedMirValue {
                        value: MirValue::Constant(_) | MirValue::TraceAccess(_),
                        ..
                    } = value.value
                    else {
                        unreachable!("expected constant or trace access, got {:?}", value);
                    };
                    args_unpacked.push(c.clone());
                } else {
                    unreachable!("expected value or parameter (or accessor on one), got {:?}", c);
                }
            }
            let vector_len = vector.children().borrow().len();
            trace_segments_arg_vector_len += vector_len;
        } else if let Some(_accessor) = child.as_accessor() {
            unreachable!("accessors should have been extracted already");
        } else {
            unreachable!("expected value, parameter, or vector of them, got {:#?}", child);
        }
    }
    (trace_segments_arg_vector_len, args_unpacked)
}
