//! Common subexpression elimination (CSE) for MIR.
//!
//! The goal is to collapse repeated MIR subtrees so the graph stays small and later passes do less
//! work. We do that by canonicalizing ops and interning identical structures via `OpInterner`,
//! with memoization to avoid revisiting nodes. The tradeoff is that we stay conservative (spans
//! and exact structural keys) to preserve correctness and diagnostics, which limits sharing.

use std::collections::HashMap;

use air_pass::Pass;
use miden_diagnostics::Spanned;

use crate::{
    CompileError,
    ir::{Link, MatchArm, Mir, MirAccessType, Op, OpInterner, Parent},
};

/// Canonicalizes MIR ops and interns identical subtrees.
pub struct Cse;

impl Cse {
    /// Construct a new CSE pass instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Cse {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for Cse {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let graph = ir.constraint_graph_mut();
        let mut state = CseState::new();
        canonicalize_graph(graph, &mut state);
        Ok(ir)
    }
}

struct CseState {
    interner: OpInterner,
    memo: HashMap<usize, Link<Op>>,
}

impl CseState {
    fn new() -> Self {
        Self {
            interner: OpInterner::new(),
            memo: HashMap::new(),
        }
    }
}

fn canonicalize_graph(graph: &mut crate::ir::Graph, state: &mut CseState) {
    {
        let mut boundary = graph.boundary_constraints_roots.borrow_mut();
        canonicalize_op_vec(&mut boundary, state);
    }
    {
        let mut integrity = graph.integrity_constraints_roots.borrow_mut();
        canonicalize_op_vec(&mut integrity, state);
    }
    for root in graph.get_function_nodes() {
        if let Some(mut function) = root.as_function_mut() {
            canonicalize_op_vec(&mut function.parameters, state);
            let new_return = canonicalize_op(function.return_type.clone(), state);
            function.return_type = new_return;
            canonicalize_linked_vec(&function.body, state);
        }
    }
    for root in graph.get_evaluator_nodes() {
        if let Some(mut evaluator) = root.as_evaluator_mut() {
            for params in evaluator.parameters.iter_mut() {
                canonicalize_op_vec(params, state);
            }
            canonicalize_linked_vec(&evaluator.body, state);
        }
    }
    for bus in graph.buses.values() {
        let mut bus = bus.borrow_mut();
        canonicalize_op_vec(&mut bus.columns, state);
        canonicalize_op_vec(&mut bus.latches, state);
    }
}

fn canonicalize_linked_vec(link: &Link<Vec<Link<Op>>>, state: &mut CseState) {
    let mut vec = link.borrow_mut();
    canonicalize_op_vec(&mut vec, state);
}

fn canonicalize_op_vec(vec: &mut [Link<Op>], state: &mut CseState) {
    for node in vec.iter_mut() {
        let new = canonicalize_op(node.clone(), state);
        *node = new;
    }
}

fn canonicalize_op(op: Link<Op>, state: &mut CseState) -> Link<Op> {
    if let Some(existing) = state.memo.get(&op.get_ptr()) {
        return existing.clone();
    }

    let snapshot = op.borrow().clone();
    let new_op = match snapshot {
        Op::Value(value) => state.interner.intern_value(value.value.clone()),
        Op::Add(add) => {
            let lhs = canonicalize_op(add.lhs.clone(), state);
            let rhs = canonicalize_op(add.rhs.clone(), state);
            state.interner.intern_add(lhs, rhs, add.span())
        },
        Op::Sub(sub) => {
            let lhs = canonicalize_op(sub.lhs.clone(), state);
            let rhs = canonicalize_op(sub.rhs.clone(), state);
            state.interner.intern_sub(lhs, rhs, sub.span())
        },
        Op::Mul(mul) => {
            let lhs = canonicalize_op(mul.lhs.clone(), state);
            let rhs = canonicalize_op(mul.rhs.clone(), state);
            state.interner.intern_mul(lhs, rhs, mul.span())
        },
        Op::Exp(exp) => {
            let lhs = canonicalize_op(exp.lhs.clone(), state);
            let rhs = canonicalize_op(exp.rhs.clone(), state);
            state.interner.intern_exp(lhs, rhs, exp.span())
        },
        Op::Vector(vector) => {
            let children = vector
                .children()
                .borrow()
                .iter()
                .cloned()
                .map(|child| canonicalize_op(child, state))
                .collect::<Vec<_>>();
            state.interner.intern_vector(children, vector.span())
        },
        Op::Matrix(matrix) => {
            let rows = matrix
                .children()
                .borrow()
                .iter()
                .cloned()
                .map(|row| canonicalize_op(row, state))
                .collect::<Vec<_>>();
            state.interner.intern_matrix(rows, matrix.span())
        },
        Op::Accessor(accessor) => {
            let indexable = canonicalize_op(accessor.indexable.clone(), state);
            let access_type = match accessor.access_type.clone() {
                MirAccessType::Default => MirAccessType::Default,
                MirAccessType::Index(index) => MirAccessType::Index(canonicalize_op(index, state)),
                MirAccessType::Matrix(row, col) => {
                    MirAccessType::Matrix(canonicalize_op(row, state), canonicalize_op(col, state))
                },
            };
            state
                .interner
                .intern_accessor(indexable, access_type, accessor.offset, accessor.span())
        },
        Op::Parameter(_) | Op::None(_) => op.clone(),
        Op::Enf(enf) => {
            let expr = canonicalize_op(enf.expr.clone(), state);
            let mut enf = op.as_enf_mut().unwrap();
            enf.expr = expr;
            op.clone()
        },
        Op::Boundary(boundary) => {
            let expr = canonicalize_op(boundary.expr.clone(), state);
            let mut boundary = op.as_boundary_mut().unwrap();
            boundary.expr = expr;
            op.clone()
        },
        Op::If(if_op) => {
            let arms = if_op
                .match_arms
                .borrow()
                .iter()
                .cloned()
                .map(|arm| MatchArm {
                    condition: canonicalize_op(arm.condition, state),
                    expr: canonicalize_op(arm.expr, state),
                })
                .collect::<Vec<_>>();
            let if_op = op.as_if_mut().unwrap();
            *if_op.match_arms.borrow_mut() = arms;
            op.clone()
        },
        Op::For(for_op) => {
            let iterators = for_op
                .iterators
                .borrow()
                .iter()
                .cloned()
                .map(|iter| canonicalize_op(iter, state))
                .collect::<Vec<_>>();
            let expr = canonicalize_op(for_op.expr.clone(), state);
            let selector = canonicalize_op(for_op.selector.clone(), state);
            let mut for_op = op.as_for_mut().unwrap();
            *for_op.iterators.borrow_mut() = iterators;
            for_op.expr = expr;
            for_op.selector = selector;
            op.clone()
        },
        Op::Call(call) => {
            let args = call
                .arguments
                .borrow()
                .iter()
                .cloned()
                .map(|arg| canonicalize_op(arg, state))
                .collect::<Vec<_>>();
            let call = op.as_call_mut().unwrap();
            *call.arguments.borrow_mut() = args;
            op.clone()
        },
        Op::Fold(fold) => {
            let iterator = canonicalize_op(fold.iterator.clone(), state);
            let initial_value = canonicalize_op(fold.initial_value.clone(), state);
            let mut fold = op.as_fold_mut().unwrap();
            fold.iterator = iterator;
            fold.initial_value = initial_value;
            op.clone()
        },
        Op::BusOp(bus_op) => {
            let args = bus_op
                .args
                .iter()
                .cloned()
                .map(|arg| canonicalize_op(arg, state))
                .collect::<Vec<_>>();
            let latch = canonicalize_op(bus_op.latch.clone(), state);
            let mut bus_op = op.as_bus_op_mut().unwrap();
            bus_op.args = args;
            bus_op.latch = latch;
            op.clone()
        },
    };

    state.memo.insert(op.get_ptr(), new_op.clone());
    new_op
}
