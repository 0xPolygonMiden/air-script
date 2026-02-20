//! MIR utility helpers.
//!
//! Includes helpers for traversing roots, stripping spans, and shareability checks
//! used by caching/interning passes.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    ops::Deref,
};

use air_parser::ast::{Identifier, NamespacedIdentifier};
use miden_diagnostics::{SourceSpan, Span};
use pretty_assertions::assert_eq;

use crate::{CompileError, ir::*, passes::Visitor};

/// Strip spans from all MIR nodes (used in tests and canonicalization).
pub fn strip_spans(mir: &mut Mir) {
    let graph = mir.constraint_graph_mut();
    let mut visitor = StripSpansVisitor::default();
    match visitor.run(graph) {
        Ok(_) => {},
        Err(e) => {
            panic!("Error stripping spans: {e:?}");
        },
    }
}

/// Visitor used by `strip_spans`.
#[derive(Default)]
pub struct StripSpansVisitor {
    _done: BTreeMap<usize, bool>,
    work_stack: Vec<Link<Node>>,
}

/// Extract selected root nodes from the graph.
pub fn extract_roots(
    graph: &Graph,
    include_boundary: bool,
    include_integrity: bool,
    include_bus: bool,
    include_func: bool,
    include_eval: bool,
) -> Vec<Link<Node>> {
    let mut nodes = Vec::new();
    if include_boundary {
        let boundary_ref = graph.boundary_constraints_roots.borrow();
        let boundary = boundary_ref.iter().map(|n| n.as_node());
        nodes.extend(boundary);
    }
    if include_integrity {
        let integrity_ref = graph.integrity_constraints_roots.borrow();
        let integrity = integrity_ref.iter().map(|n| n.as_node());
        nodes.extend(integrity);
    }
    if include_bus {
        let buses = graph.get_bus_nodes();
        let bus_columns =
            buses.iter().flat_map(|b| b.borrow().columns.clone()).map(|n| n.as_node());
        let bus_latches =
            buses.iter().flat_map(|b| b.borrow().latches.clone()).map(|n| n.as_node());
        nodes.extend(bus_columns);
        nodes.extend(bus_latches);
    }
    if include_func {
        let funcs = graph.get_function_nodes().into_iter().map(|n| n.as_node());
        nodes.extend(funcs);
    }
    if include_eval {
        let evals = graph.get_evaluator_nodes().into_iter().map(|n| n.as_node());
        nodes.extend(evals);
    }
    nodes
}

/// Extract all root nodes from the graph.
pub fn extract_all_roots(graph: &Graph) -> Vec<Link<Node>> {
    extract_roots(graph, true, true, true, true, true)
}

/// Extract boundary constraint roots.
pub fn extract_boundary_roots(graph: &Graph) -> Vec<Link<Node>> {
    extract_roots(graph, true, false, false, false, false)
}

/// Extract integrity constraint roots.
pub fn extract_integrity_roots(graph: &Graph) -> Vec<Link<Node>> {
    extract_roots(graph, false, true, false, false, false)
}

/// Extract bus constraint roots.
pub fn extract_bus_roots(graph: &Graph) -> Vec<Link<Node>> {
    extract_roots(graph, false, false, true, false, false)
}

/// Extract function roots.
pub fn extract_function_roots(graph: &Graph) -> Vec<Link<Node>> {
    extract_roots(graph, false, false, false, true, false)
}

/// Extract evaluator roots.
pub fn extract_evaluator_roots(graph: &Graph) -> Vec<Link<Node>> {
    extract_roots(graph, false, false, false, false, true)
}

/// Conservative shareability predicate for ops.
///
/// Used to gate caching/interning to avoid accidental semantic changes.
pub fn is_shareable_op(op: &Op) -> bool {
    matches!(
        op,
        Op::Value(_)
            | Op::Parameter(_)
            | Op::Add(_)
            | Op::Sub(_)
            | Op::Mul(_)
            | Op::Exp(_)
            | Op::Vector(_)
            | Op::Matrix(_)
            | Op::Accessor(_)
    )
}

/// Conservative shareability predicate for roots.
///
/// Only functions with fully shareable bodies are considered shareable.
pub fn is_shareable_root(root: &Link<Root>) -> bool {
    let mut memo = HashMap::new();
    let mut visiting = HashSet::new();
    is_shareable_root_inner(root, &mut memo, &mut visiting)
}

fn is_shareable_root_inner(
    root: &Link<Root>,
    memo: &mut HashMap<usize, bool>,
    visiting: &mut HashSet<usize>,
) -> bool {
    let root_ptr = root.get_ptr();
    if let Some(result) = memo.get(&root_ptr) {
        return *result;
    }
    if !visiting.insert(root_ptr) {
        // Cycles are not shareable.
        return false;
    }

    let body = {
        let root_ref = root.borrow();
        match &*root_ref {
            Root::Function(f) => f.body.clone(),
            _ => {
                memo.insert(root_ptr, false);
                visiting.remove(&root_ptr);
                return false;
            },
        }
    };

    let mut stack: Vec<Link<Op>> = body.borrow().iter().cloned().collect();
    let mut seen = HashSet::new();

    while let Some(op) = stack.pop() {
        let ptr = op.get_ptr();
        if !seen.insert(ptr) {
            continue;
        }
        let op_ref = op.borrow();
        match &*op_ref {
            Op::Call(call) => {
                let callee = call.function.clone();
                let is_callee_shareable = matches!(*callee.borrow(), Root::Function(_))
                    && is_shareable_root_inner(&callee, memo, visiting);
                if !is_callee_shareable {
                    memo.insert(root_ptr, false);
                    visiting.remove(&root_ptr);
                    return false;
                }
            },
            _ => {
                if !is_shareable_op(&op_ref) {
                    memo.insert(root_ptr, false);
                    visiting.remove(&root_ptr);
                    return false;
                }
            },
        }
        for child in op_ref.children().borrow().iter() {
            stack.push(child.clone());
        }
    }

    visiting.remove(&root_ptr);
    memo.insert(root_ptr, true);
    true
}

#[derive(Default, Debug, Clone)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_ops: usize,
    pub total_roots: usize,
    pub counts: BTreeMap<&'static str, usize>,
}

impl GraphStats {
    fn record_root(&mut self, name: &'static str) {
        self.total_nodes += 1;
        self.total_roots += 1;
        *self.counts.entry(name).or_default() += 1;
    }
    fn record_op(&mut self, name: &'static str) {
        self.total_nodes += 1;
        self.total_ops += 1;
        *self.counts.entry(name).or_default() += 1;
    }
    pub fn log(&self, label: &str) {
        let mut parts = Vec::new();
        for (name, count) in &self.counts {
            parts.push(format!("{name}={count}"));
        }
        if parts.is_empty() {
            eprintln!(
                "mir: stats {label} nodes={} ops={} roots={}",
                self.total_nodes, self.total_ops, self.total_roots
            );
        } else {
            eprintln!(
                "mir: stats {label} nodes={} ops={} roots={} {}",
                self.total_nodes,
                self.total_ops,
                self.total_roots,
                parts.join(" ")
            );
        }
    }
}

pub fn collect_graph_stats(graph: &Graph) -> GraphStats {
    let mut stats = GraphStats::default();
    let mut seen = HashSet::new();
    let mut stack = extract_all_roots(graph);

    while let Some(node) = stack.pop() {
        if node.is_stale() {
            continue;
        }
        let ptr = node.get_ptr();
        if !seen.insert(ptr) {
            continue;
        }

        match &*node.borrow() {
            Node::Function(_) => stats.record_root("Function"),
            Node::Evaluator(_) => stats.record_root("Evaluator"),
            Node::Enf(_) => stats.record_op("Enf"),
            Node::Boundary(_) => stats.record_op("Boundary"),
            Node::Add(_) => stats.record_op("Add"),
            Node::Sub(_) => stats.record_op("Sub"),
            Node::Mul(_) => stats.record_op("Mul"),
            Node::Exp(_) => stats.record_op("Exp"),
            Node::If(_) => stats.record_op("If"),
            Node::For(_) => stats.record_op("For"),
            Node::Call(_) => stats.record_op("Call"),
            Node::Fold(_) => stats.record_op("Fold"),
            Node::Vector(_) => stats.record_op("Vector"),
            Node::Matrix(_) => stats.record_op("Matrix"),
            Node::Accessor(_) => stats.record_op("Accessor"),
            Node::BusOp(_) => stats.record_op("BusOp"),
            Node::Parameter(_) => stats.record_op("Parameter"),
            Node::Value(_) => stats.record_op("Value"),
            Node::None(_) => {},
        }

        if node.as_owner().is_some() {
            for child in node.children().borrow().iter() {
                stack.push(child.clone().as_node());
            }
        }
    }

    stats
}

pub fn log_graph_stats_if_enabled(graph: &Graph, label: &str) {
    if std::env::var("AIR_MIR_STATS").is_ok() {
        let stats = collect_graph_stats(graph);
        stats.log(label);
    }
}

/// Debug helper to locate parameters with invalid owner ids.
/// Enabled by setting AIR_DEBUG_INVALID_PARAMS.
pub fn debug_invalid_params(graph: &Graph, label: &str) {
    if std::env::var("AIR_DEBUG_INVALID_PARAMS").is_err() {
        return;
    }

    let mut seen: HashSet<usize> = HashSet::new();
    let mut path: Vec<&'static str> = Vec::new();
    let mut found = 0usize;

    for root in extract_all_roots(graph) {
        debug_invalid_params_dfs(root, &mut path, &mut seen, &mut found);
    }

    if found > 0 {
        eprintln!("mir: invalid params after {label}: {found}");
    }
}

/// Debug helper to locate any parameters reachable from constraint roots.
/// Enabled by setting AIR_DEBUG_PARAMS.
pub fn debug_params(graph: &Graph, label: &str) {
    if std::env::var("AIR_DEBUG_PARAMS").is_err() {
        return;
    }

    let mut seen: HashSet<usize> = HashSet::new();
    let mut path: Vec<&'static str> = Vec::new();
    let mut found = 0usize;

    for root in extract_all_roots(graph) {
        debug_params_dfs(root, &mut path, &mut seen, &mut found);
    }

    if found > 0 {
        eprintln!("mir: params after {label}: {found}");
    }
}

fn debug_invalid_params_dfs(
    node: Link<Node>,
    path: &mut Vec<&'static str>,
    seen: &mut HashSet<usize>,
    found: &mut usize,
) {
    let ptr = node.get_ptr();
    if !seen.insert(ptr) {
        return;
    }

    let kind = match node.borrow().deref() {
        Node::Function(_) => "Function",
        Node::Evaluator(_) => "Evaluator",
        Node::Enf(_) => "Enf",
        Node::Boundary(_) => "Boundary",
        Node::Add(_) => "Add",
        Node::Sub(_) => "Sub",
        Node::Mul(_) => "Mul",
        Node::Exp(_) => "Exp",
        Node::If(_) => "If",
        Node::For(_) => "For",
        Node::Call(_) => "Call",
        Node::Fold(_) => "Fold",
        Node::Vector(_) => "Vector",
        Node::Matrix(_) => "Matrix",
        Node::Accessor(_) => "Accessor",
        Node::BusOp(_) => "BusOp",
        Node::Parameter(_) => "Parameter",
        Node::Value(_) => "Value",
        Node::None(_) => "None",
    };
    path.push(kind);

    if let Node::Parameter(param_back) = &*node.borrow() {
        if let Some(param_op) = param_back.to_link() {
            if let Some(param_ref) = param_op.as_parameter() {
                if param_ref.owner_id.is_unknown() {
                    *found += 1;
                    eprintln!("mir: invalid param path={:?} param={:?}", path, *param_ref);
                }
            }
        }
    }

    if node.as_owner().is_some() {
        for child in node.children().borrow().iter() {
            debug_invalid_params_dfs(child.clone().as_node(), path, seen, found);
        }
    }

    path.pop();
}

fn debug_params_dfs(
    node: Link<Node>,
    path: &mut Vec<&'static str>,
    seen: &mut HashSet<usize>,
    found: &mut usize,
) {
    let ptr = node.get_ptr();
    if !seen.insert(ptr) {
        return;
    }

    let kind = match node.borrow().deref() {
        Node::Function(_) => "Function",
        Node::Evaluator(_) => "Evaluator",
        Node::Enf(_) => "Enf",
        Node::Boundary(_) => "Boundary",
        Node::Add(_) => "Add",
        Node::Sub(_) => "Sub",
        Node::Mul(_) => "Mul",
        Node::Exp(_) => "Exp",
        Node::If(_) => "If",
        Node::For(_) => "For",
        Node::Call(_) => "Call",
        Node::Fold(_) => "Fold",
        Node::Vector(_) => "Vector",
        Node::Matrix(_) => "Matrix",
        Node::Accessor(_) => "Accessor",
        Node::BusOp(_) => "BusOp",
        Node::Parameter(_) => "Parameter",
        Node::Value(_) => "Value",
        Node::None(_) => "None",
    };
    path.push(kind);

    if let Node::Parameter(param_back) = &*node.borrow() {
        if let Some(param_op) = param_back.to_link() {
            if let Some(param_ref) = param_op.as_parameter() {
                *found += 1;
                eprintln!(
                    "mir: param path={:?} ptr={} param={:?} owner_id={:?}",
                    path,
                    param_op.get_ptr(),
                    *param_ref,
                    param_ref.owner_id
                );
            }
        }
    }

    if node.as_owner().is_some() {
        for child in node.children().borrow().iter() {
            debug_params_dfs(child.clone().as_node(), path, seen, found);
        }
    }

    path.pop();
}

impl Visitor for StripSpansVisitor {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn root_nodes_to_visit(&self, graph: &Graph) -> Vec<Link<Node>> {
        extract_all_roots(graph)
    }

    fn visit_function(
        &mut self,
        _graph: &mut Graph,
        function: Link<Root>,
    ) -> Result<(), CompileError> {
        let mut function = function.as_function_mut().unwrap();
        function.span = Default::default();
        Ok(())
    }

    fn visit_evaluator(
        &mut self,
        _graph: &mut Graph,
        evaluator: Link<Root>,
    ) -> Result<(), CompileError> {
        let mut evaluator = evaluator.as_evaluator_mut().unwrap();
        evaluator.span = Default::default();
        Ok(())
    }

    fn visit_enf(&mut self, _graph: &mut Graph, enf: Link<Op>) -> Result<(), CompileError> {
        let mut enf = enf.as_enf_mut().unwrap();
        enf.span = Default::default();
        Ok(())
    }

    fn visit_boundary(
        &mut self,
        _graph: &mut Graph,
        boundary: Link<Op>,
    ) -> Result<(), CompileError> {
        let mut boundary = boundary.as_boundary_mut().unwrap();
        boundary.span = Default::default();
        Ok(())
    }

    fn visit_add(&mut self, _graph: &mut Graph, add: Link<Op>) -> Result<(), CompileError> {
        let mut add = add.as_add_mut().unwrap();
        add.span = Default::default();
        Ok(())
    }

    fn visit_sub(&mut self, _graph: &mut Graph, sub: Link<Op>) -> Result<(), CompileError> {
        let mut sub = sub.as_sub_mut().unwrap();
        sub.span = Default::default();
        Ok(())
    }

    fn visit_mul(&mut self, _graph: &mut Graph, mul: Link<Op>) -> Result<(), CompileError> {
        let mut mul = mul.as_mul_mut().unwrap();
        mul.span = Default::default();
        Ok(())
    }

    fn visit_exp(&mut self, _graph: &mut Graph, exp: Link<Op>) -> Result<(), CompileError> {
        let mut exp = exp.as_exp_mut().unwrap();
        exp.span = Default::default();
        Ok(())
    }

    fn visit_if(&mut self, _graph: &mut Graph, if_node: Link<Op>) -> Result<(), CompileError> {
        let mut if_node = if_node.as_if_mut().unwrap();
        if_node.span = Default::default();
        Ok(())
    }

    fn visit_for(&mut self, _graph: &mut Graph, for_node: Link<Op>) -> Result<(), CompileError> {
        let mut for_node = for_node.as_for_mut().unwrap();
        for_node.span = Default::default();
        Ok(())
    }

    fn visit_call(&mut self, _graph: &mut Graph, call: Link<Op>) -> Result<(), CompileError> {
        let mut call = call.as_call_mut().unwrap();
        call.span = Default::default();
        Ok(())
    }

    fn visit_fold(&mut self, _graph: &mut Graph, fold: Link<Op>) -> Result<(), CompileError> {
        let mut fold = fold.as_fold_mut().unwrap();
        fold.span = Default::default();
        Ok(())
    }

    fn visit_vector(&mut self, _graph: &mut Graph, vector: Link<Op>) -> Result<(), CompileError> {
        let mut vector = vector.as_vector_mut().unwrap();
        vector.span = Default::default();
        Ok(())
    }

    fn visit_matrix(&mut self, _graph: &mut Graph, matrix: Link<Op>) -> Result<(), CompileError> {
        let mut matrix = matrix.as_matrix_mut().unwrap();
        matrix.span = Default::default();
        Ok(())
    }

    fn visit_accessor(
        &mut self,
        _graph: &mut Graph,
        accessor: Link<Op>,
    ) -> Result<(), CompileError> {
        let mut accessor = accessor.as_accessor_mut().unwrap();
        accessor.span = Default::default();
        Ok(())
    }

    fn visit_bus_op(&mut self, _graph: &mut Graph, bus_op: Link<Op>) -> Result<(), CompileError> {
        let mut bus_op = bus_op.as_bus_op_mut().unwrap();
        bus_op.span = Default::default();
        Ok(())
    }

    fn visit_parameter(
        &mut self,
        _graph: &mut Graph,
        parameter: Link<Op>,
    ) -> Result<(), CompileError> {
        let mut parameter = parameter.as_parameter_mut().unwrap();
        parameter.span = Default::default();
        Ok(())
    }

    fn visit_value(&mut self, _graph: &mut Graph, value: Link<Op>) -> Result<(), CompileError> {
        let mut value = value.as_value_mut().unwrap();
        value.value.span = Default::default();
        match &mut value.value.value {
            MirValue::Constant(_) => {},
            MirValue::TraceAccess(_) => {},
            MirValue::PeriodicColumn(v) => {
                v.name.module.0 = Span::new(SourceSpan::default(), v.name.module.0.item.clone());
                match v.name.item {
                    NamespacedIdentifier::Function(f) => {
                        v.name.item = NamespacedIdentifier::Function(Identifier::new(
                            SourceSpan::default(),
                            f.0.item,
                        ));
                    },
                    NamespacedIdentifier::Binding(b) => {
                        v.name.item = NamespacedIdentifier::Binding(Identifier::new(
                            SourceSpan::default(),
                            b.0.item,
                        ));
                    },
                };
            },
            MirValue::PublicInput(v) => {
                v.name.0 = Span::new(SourceSpan::default(), v.name.0.item);
            },
            MirValue::PublicInputTable(v) => {
                v.table_name.0 = Span::new(SourceSpan::default(), v.table_name.0.item);
            },
            MirValue::RandomValue(_) => {},
            MirValue::TraceAccessBinding(_) => {},
            MirValue::BusAccess(_) => {},
            MirValue::Null | MirValue::Unconstrained => {},
        }
        Ok(())
    }
}

pub fn hash<T: Hash>(val: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}

fn extract_and_compare_mir(
    lhs: &mut Mir,
    rhs: &mut Mir,
    extract: impl Fn(&Graph) -> Vec<Link<Node>>,
) -> bool {
    strip_spans(lhs);
    strip_spans(rhs);
    let lhs = extract(lhs.constraint_graph());
    let rhs = extract(rhs.constraint_graph());
    hash(&lhs) == hash(&rhs)
}

pub fn compare_mir(lhs: &mut Mir, rhs: &mut Mir) -> bool {
    extract_and_compare_mir(lhs, rhs, extract_all_roots)
}

pub fn compare_boundary(lhs: &mut Mir, rhs: &mut Mir) {
    extract_and_compare_mir(lhs, rhs, extract_boundary_roots);
}

pub fn compare_integrity(lhs: &mut Mir, rhs: &mut Mir) {
    extract_and_compare_mir(lhs, rhs, extract_integrity_roots);
}

pub fn compare_bus(lhs: &mut Mir, rhs: &mut Mir) {
    extract_and_compare_mir(lhs, rhs, extract_bus_roots);
}

pub fn compare_function(lhs: &mut Mir, rhs: &mut Mir) {
    extract_and_compare_mir(lhs, rhs, extract_function_roots);
}

pub fn compare_evaluator(lhs: &mut Mir, rhs: &mut Mir) {
    extract_and_compare_mir(lhs, rhs, extract_evaluator_roots);
}

fn extract_and_assert_mir_eq(
    lhs: &mut Mir,
    rhs: &mut Mir,
    extract: impl Fn(&Graph) -> Vec<Link<Node>>,
) {
    strip_spans(lhs);
    strip_spans(rhs);
    let lhs = extract(lhs.constraint_graph());
    let rhs = extract(rhs.constraint_graph());
    assert_eq!(lhs, rhs);
}

pub fn assert_mir_eq(lhs: &mut Mir, rhs: &mut Mir) {
    extract_and_assert_mir_eq(lhs, rhs, extract_all_roots);
}

pub fn assert_boundary_eq(lhs: &mut Mir, rhs: &mut Mir) {
    extract_and_assert_mir_eq(lhs, rhs, extract_boundary_roots);
}

pub fn assert_integrity_eq(lhs: &mut Mir, rhs: &mut Mir) {
    extract_and_assert_mir_eq(lhs, rhs, extract_integrity_roots);
}

pub fn assert_bus_eq(lhs: &mut Mir, rhs: &mut Mir) {
    extract_and_assert_mir_eq(lhs, rhs, extract_bus_roots);
}

pub fn assert_function_eq(lhs: &mut Mir, rhs: &mut Mir) {
    extract_and_assert_mir_eq(lhs, rhs, extract_function_roots);
}

pub fn assert_evaluator_eq(lhs: &mut Mir, rhs: &mut Mir) {
    extract_and_assert_mir_eq(lhs, rhs, extract_evaluator_roots);
}
