use std::ops::Deref;

use miden_diagnostics::{DiagnosticsHandler, Spanned};

use crate::{
    CompileError,
    ir::{Graph, Link, Node, Op, RandomInputs, Vector},
    passes::{
        Visitor,
        unrolling::{
            match_optimizer::MatchOptimizer,
            unrolling_first_pass::{
                visit_add_bis, visit_boundary_bis, visit_enf_bis, visit_exp_bis, visit_fold_bis,
                visit_mul_bis, visit_sub_bis, visit_value_bis, visit_vector_bis,
            },
        },
    },
};

pub struct UnrollingThirdPass<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,
    // current evaluations of nodes at random points
    random_inputs: RandomInputs,
}

impl<'a> UnrollingThirdPass<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self {
            diagnostics,
            work_stack: vec![],
            random_inputs: RandomInputs::default(),
        }
    }
}

// For the first pass of Unrolling, we use a tweaked version of the Visitor trait,
// each visit_*_bis function returns an Option<Link<Op>> instead of Result<(), CompileError>,
// to mutate the nodes (e.g. modifying a Operation<Vectors> to Vector<Operations>)
impl UnrollingThirdPass<'_> {
    /// Visiting an `If` node consists of evaluating all the main trace constraints contained in
    /// the match arms, and combining them to optimize the resulting vector of constraints if
    /// possible. We handle bus related constraints separately, as they cannot be combined with
    /// main trace constraints.
    fn visit_if_bis(&mut self, if_node: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
        let if_ref = if_node.as_if().unwrap();
        let match_arms = if_ref.match_arms.borrow();

        // 1. Instantiate a new MatchOptimizer to handle the constraints of this node
        let mut match_optimizer = MatchOptimizer::new(&mut self.random_inputs);

        let mut bus_related_constraints = Vec::new();

        // 2. For each match arm, gather bus-related constraints
        // to be handled separately and evaluate the main constraints
        for match_arm in match_arms.iter() {
            let bus_related_constraints_for_match_arm =
                match_optimizer.evaluate_match_arm(match_arm)?;
            bus_related_constraints
                .push((match_arm.condition.clone(), bus_related_constraints_for_match_arm));
        }

        // 3. Construct the new vector of combined main constraints
        let combined_main_constraints = match_optimizer.reduce_main_constraints(if_ref.span);

        // 4. Add all the constraints that are bus-related
        let new_vec = MatchOptimizer::gather_all_constraints(
            &mut bus_related_constraints,
            combined_main_constraints,
            if_ref.span,
        );

        Ok(Some(Vector::create(new_vec, if_ref.span())))
    }
}

impl Visitor for UnrollingThirdPass<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }

    // We visit all boundary constraints and all integrity constraints
    // No need to visit the functions or evaluators, as they should have been inlined before this
    // pass
    fn root_nodes_to_visit(&self, graph: &Graph) -> Vec<Link<Node>> {
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
            .chain(bus_roots.into_iter().map(|b| b.as_node()));
        combined_roots.collect()
    }

    fn visit_node(&mut self, _graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        // In this pass, we both need to dispatch the visitor depending on the node type,
        // and also mutate the node if needed. We implement custom visit_*_bis methods
        // that returns a Some(updated_node) if we need to update the node's value.
        let updated_op: Result<Option<Link<Op>>, CompileError> = match node.borrow().deref() {
            Node::Enf(e) => e.to_link().map_or(Ok(None), visit_enf_bis),
            Node::Boundary(b) => b.to_link().map_or(Ok(None), visit_boundary_bis),
            Node::Add(a) => a.to_link().map_or(Ok(None), visit_add_bis),
            Node::Sub(s) => s.to_link().map_or(Ok(None), visit_sub_bis),
            Node::Mul(m) => m.to_link().map_or(Ok(None), visit_mul_bis),
            Node::Exp(e) => e.to_link().map_or(Ok(None), visit_exp_bis),
            Node::Fold(f) => f.to_link().map_or(Ok(None), visit_fold_bis),
            Node::Vector(v) => v.to_link().map_or(Ok(None), visit_vector_bis),
            Node::Value(v) => v.to_link().map_or(Ok(None), visit_value_bis),
            Node::If(i) => i.to_link().map_or(Ok(None), |el| self.visit_if_bis(el)),
            Node::Accessor(_a) => Ok(None),
            Node::BusOp(_b) => Ok(None),
            Node::Matrix(_) => Ok(None), // Matrix are already unrolled, we have nothing to do
            Node::None(_) => Ok(None),
            Node::Function(_)
            | Node::Evaluator(_)
            | Node::Call(_)
            | Node::For(_)
            | Node::Parameter(_) => {
                unreachable!(
                    "Unexpected node during Unrolling: Function, Evaluators, Calls, For nodes and Parameters should have been inlined before this pass. Found: {:?}",
                    node
                );
            },
        };

        // We update the node if needed
        if let Some(updated_op) = updated_op? {
            node.as_op().unwrap().set(&updated_op);
        }

        Ok(())
    }
}
