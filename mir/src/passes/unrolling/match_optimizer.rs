use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
};

use miden_diagnostics::SourceSpan;

use crate::{
    CompileError,
    ir::{
        Add, ConstantValue, Link, MatchArm, MirValue, Mul, Op, Parent, QuadFelt, RandomInputs,
        SpannedMirValue, Sub, Value,
    },
    passes::duplicate_node,
};

type ConstraintEvaluationMap = BTreeMap<usize, Vec<(Link<Op>, Link<Op>)>>;

/// This struct provides methods used to combine and optimize constraints contained in match
/// statements (corresponding to `If` nodes in the MIR)
#[derive(Debug)]
pub struct MatchOptimizer<'a> {
    // A reference to the random inputs used to evaluate the constraints
    // (common for all matches)
    random_inputs: &'a mut RandomInputs,
    // Used to keep track of the evaluations values we've seen so far
    // in the current `If` node.
    node_evals: Vec<QuadFelt>,
    // Used to keep track of the evaluation of each constraint
    // We use indices to the node_evals vector as keys, to avoid non-determinism for
    // iterating across random evaluations.
    constraints_evaluation_indices: ConstraintEvaluationMap,
}

impl<'a> MatchOptimizer<'a> {
    /// Instantiates a new `MatchOptimizer` with the given `RandomInputs`.
    pub fn new(random_inputs: &'a mut RandomInputs) -> Self {
        Self {
            random_inputs,
            node_evals: Vec::new(),
            constraints_evaluation_indices: ConstraintEvaluationMap::new(),
        }
    }

    /// For a given match arm, splits main and bus-related constraints, and evaluates the main
    /// constraints
    pub fn evaluate_match_arm(
        &mut self,
        match_arm: &MatchArm,
    ) -> Result<Vec<Link<Op>>, CompileError> {
        let condition = match_arm.condition.clone();
        let expr = match_arm.expr.clone();

        // 1.1. Get all the individual constraints corresponding to this arm
        let all_constraints = if let Op::Vector(expr_vector) = expr.borrow().deref() {
            expr_vector.children().borrow().deref().clone()
        } else {
            vec![expr.clone()]
        };
        let mut constraints_to_eval = Vec::new();
        let mut bus_related_constraints_for_match_arm = Vec::new();

        // 1.2. Filter out all BusOp nodes from the constraints, we will handle them separately
        for constraint in all_constraints {
            match constraint.borrow().deref() {
                Op::BusOp(_) => bus_related_constraints_for_match_arm.push(constraint.clone()),
                Op::Enf(enf) => match enf.expr.borrow().deref() {
                    Op::BusOp(_) => bus_related_constraints_for_match_arm.push(enf.expr.clone()),
                    _ => constraints_to_eval.push(constraint.clone()),
                },
                _ => constraints_to_eval.push(constraint.clone()),
            }
        }

        // 1.3. Evaluate all the other constraints at random points
        for constraint in constraints_to_eval {
            let eval = self.random_inputs.eval(constraint.clone())?;
            // Check if we already have this eval in our list
            match self.node_evals.iter().position(|e| e == &eval) {
                Some(index) => {
                    // If the eval is already in the list, we use its index in the
                    // node_evals vec
                    let Some(constraints_vec) = self.constraints_evaluation_indices.get_mut(&index)
                    else {
                        unreachable!(
                            "Error: Reference to an evaluation index not found in constraints_evaluation_indices"
                        );
                    };
                    // We add the constraint to the corresponding vector only if the
                    // condition is not already present
                    // to remove duplicate constraints for the same selector
                    if !constraints_vec.iter().any(|(c, _)| c == &condition) {
                        constraints_vec.push((condition.clone(), constraint.clone()))
                    }
                },
                None => {
                    // Otherwise, we add it to the list and use its index
                    self.node_evals.push(eval);
                    let index = self.node_evals.len() - 1;
                    if self
                        .constraints_evaluation_indices
                        .insert(index, vec![(condition.clone(), constraint.clone())])
                        .is_some()
                    {
                        unreachable!(
                            "Error: A new evaluation index was found in constraints_evaluation_indices"
                        );
                    }
                },
            }
        }
        Ok(bus_related_constraints_for_match_arm)
    }

    /// For each evaluation, computes the number of constraints (with different selectors)
    /// evaluating to it.
    // Returns eval_lens: BTreeMap<len, Vec<evaluation_index>>
    pub fn compute_eval_lens(&self) -> BTreeMap<usize, Vec<usize>> {
        let mut eval_lens = BTreeMap::new();
        for (eval_index, constraints_vec) in self.constraints_evaluation_indices.iter() {
            eval_lens
                .entry(constraints_vec.len())
                .and_modify(|v: &mut Vec<_>| v.push(*eval_index))
                .or_insert(vec![*eval_index]);
        }
        eval_lens
    }

    /// Combines the constraints in an optimized way, based on their evaluations.
    pub fn reduce_main_constraints(&mut self, span: SourceSpan) -> Vec<Link<Op>> {
        let mut new_vec = vec![];

        let mut eval_lens = self.compute_eval_lens();

        // Note: this will always terminate as we always remove at least one constraint per
        // iteration
        while !self.constraints_evaluation_indices.is_empty() {
            let mut new_constraint = vec![];
            let mut taken_eval_indices = vec![];

            // Start picking constraints from the biggest sets that evaluate to the same values
            for eval_index in eval_lens.values().cloned().rev().flatten() {
                let constraints = self.constraints_evaluation_indices.get(&eval_index).unwrap();

                if have_disjoint_conditions(new_constraint.clone(), constraints) {
                    // If the new constraints are disjoint from the current ones, we can add
                    // them
                    new_constraint.push(constraints.clone());
                    taken_eval_indices.push(eval_index);
                }
            }

            // Remove the picked evaluation indices from all structures
            for eval_index in taken_eval_indices {
                self.constraints_evaluation_indices.remove(&eval_index);
                eval_lens.iter_mut().for_each(|(_len, evals)| {
                    evals.retain(|&e_idx| e_idx != eval_index);
                });
                eval_lens.retain(|_, constraints| !constraints.is_empty());
            }

            // Create combined constraint
            let mut cur_node = None;
            for equivalent_constraints in new_constraint {
                let mut cur_condition = None;
                for (condition, _) in equivalent_constraints.clone() {
                    if cur_condition.is_none() {
                        cur_condition = Some(condition.clone());
                    } else {
                        cur_condition =
                            Some(Add::create(cur_condition.unwrap(), condition.clone(), span));
                    }
                }
                let new_node = Mul::create(
                    cur_condition.unwrap(),
                    equivalent_constraints.first().unwrap().1.clone(),
                    span,
                );

                cur_node = match cur_node {
                    Some(existing) => Some(Add::create(existing, new_node, span)),
                    None => Some(new_node),
                };
            }

            // Note: This will ensure the resulting constraint is in the form `Sub(x,y)`
            // representing `enf x = y`
            let zero_node = Value::create(SpannedMirValue {
                span: Default::default(),
                value: MirValue::Constant(ConstantValue::Felt(0)),
            });
            // The following unwrap is safe as we always have at least one constraint above
            let new_node_with_sub_zero = Sub::create(cur_node.unwrap(), zero_node, span);

            new_vec.push(new_node_with_sub_zero);
        }
        new_vec
    }

    /// Adds bus-related constraints to the resulting constraints vector.
    pub fn gather_all_constraints(
        bus_related_constraints: &mut [(Link<Op>, Vec<Link<Op>>)],
        combined_main_constraints: Vec<Link<Op>>,
        span: SourceSpan,
    ) -> Vec<Link<Op>> {
        let mut all_constraints = combined_main_constraints;
        for (condition, constraints) in bus_related_constraints.iter_mut() {
            for constraint in constraints.iter_mut() {
                let cur_latch = constraint.as_bus_op().unwrap().latch.clone();
                let new_latch = Mul::create(
                    condition.clone(),
                    duplicate_node(cur_latch, &mut HashMap::new()),
                    span,
                );
                constraint
                    .as_bus_op_mut()
                    .unwrap()
                    .latch
                    .borrow_mut()
                    .clone_from(&new_latch.borrow());
                all_constraints.push(constraint.clone());
            }
        }
        all_constraints
    }
}

/// Helper function used to check whether two sets of constraints have disjoint conditions.
/// This is used to combine constraints in the `visit_if_bis` method, as we can only combine
/// constraints that have disjoint selectors.
fn have_disjoint_conditions(
    cur_constraints: Vec<Vec<(Link<Op>, Link<Op>)>>,
    new_constraints: &[(Link<Op>, Link<Op>)],
) -> bool {
    for (condition, _) in new_constraints {
        if cur_constraints.iter().flatten().any(|(c, _)| c == condition) {
            return false; // Duplicate condition found
        }
    }
    true
}
