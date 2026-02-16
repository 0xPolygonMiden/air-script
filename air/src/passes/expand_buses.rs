use std::collections::BTreeMap;

use air_parser::ast::{Boundary, BusConstraintForm, BusType, TraceSegmentId};
use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;
use mir::ir::BusOpKind;

use crate::{
    Air, BusBoundary, BusOp, CompileError, ConstraintDomain, NodeIndex, Operation, TraceAccess,
};

pub struct BusOpExpand<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,
}

impl Pass for BusOpExpand<'_> {
    type Input<'a> = Air;
    type Output<'a> = Air;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let buses = ir.buses.clone();

        for (bus_index, (_ident, bus)) in buses.iter().enumerate() {
            let bus_type = bus.bus_type;

            // Expand bus boundary constraints first
            self.handle_boundary_constraint(
                &mut ir,
                bus_type,
                &bus.first,
                bus.first_tag,
                Boundary::First,
                bus_index,
            );
            self.handle_boundary_constraint(
                &mut ir,
                bus_type,
                &bus.last,
                bus.last_tag,
                Boundary::Last,
                bus_index,
            );

            let bus_ops = bus.bus_ops.clone();

            let bus_trace_access = TraceAccess::new(TraceSegmentId::Aux, bus_index, 0);
            let bus_trace_access_with_offset = TraceAccess::new(TraceSegmentId::Aux, bus_index, 1);

            let bus_access = ir
                .constraint_graph_mut()
                .insert_node(Operation::Value(crate::Value::TraceAccess(bus_trace_access)));
            let bus_access_with_offset =
                ir.constraint_graph_mut()
                    .insert_node(Operation::Value(crate::Value::TraceAccess(
                        bus_trace_access_with_offset,
                    )));

            // Then, depending on the bus type, expand the integrity constraint if
            // the bus is constrained
            if !bus_ops.is_empty() {
                match bus_type {
                    BusType::Multiset => {
                        self.expand_multiset_constraint(
                            &mut ir,
                            bus_ops,
                            bus_access,
                            bus_access_with_offset,
                            bus_index,
                            bus.transition_tag,
                            bus.constraint_form,
                        );
                    },
                    BusType::Logup => {
                        self.expand_logup_constraint(
                            &mut ir,
                            bus_ops,
                            bus_access,
                            bus_access_with_offset,
                            bus_index,
                            bus.transition_tag,
                        );
                    },
                }
            }
        }

        ir.num_random_values = buses
            .values()
            .map(|bus| bus.bus_ops.iter().map(|a| a.columns.len() + 1).max().unwrap_or_default())
            .max()
            .unwrap_or_default() as u16;

        Ok(ir)
    }
}

impl<'a> BusOpExpand<'a> {
    #[allow(unused)]
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }

    /// Helper function to handle and insert into the graph a bus boundary constraint if possible
    fn handle_boundary_constraint(
        &self,
        ir: &mut Air,
        bus_type: BusType,
        bus_boundary: &BusBoundary,
        tag: Option<u64>,
        boundary: Boundary,
        bus_index: usize,
    ) {
        let value = match bus_boundary {
            // Boundaries to PublicInputTable reference a value corresponding to the random
            // reduction of a public input table for a given bus type (multiset or logUp)
            BusBoundary::PublicInputTable(public_input_table_access) => {
                ir.constraint_graph_mut().insert_node(Operation::Value(
                    crate::Value::PublicInputTable(*public_input_table_access),
                ))
            },
            BusBoundary::Null => {
                // The value of the constraint for an empty bus depends on the bus types (1 for
                // multiset, 0 for logup)
                match bus_type {
                    BusType::Multiset => ir
                        .constraint_graph_mut()
                        .insert_node(Operation::Value(crate::Value::Constant(1))),
                    BusType::Logup => ir
                        .constraint_graph_mut()
                        .insert_node(Operation::Value(crate::Value::Constant(0))),
                }
            },
            // Unconstrained boundaries do not require any constraints
            BusBoundary::Unconstrained => return,
        };
        let bus_trace_access = TraceAccess::new(TraceSegmentId::Aux, bus_index, 0);
        let bus_access = ir
            .constraint_graph_mut()
            .insert_node(Operation::Value(crate::Value::TraceAccess(bus_trace_access)));

        // Then, we enforce for instance the constraint `p.first = 0/1` or `q.first = value` to
        // have an empty bus initially or equal to the values given in a public input table.
        let root = ir.constraint_graph_mut().insert_node(Operation::Sub(bus_access, value));
        let domain = match boundary {
            Boundary::First => ConstraintDomain::FirstRow,
            Boundary::Last => ConstraintDomain::LastRow,
        };
        // Store the generated constraint
        ir.constraints.insert_constraint(TraceSegmentId::Aux, root, domain, tag);

        // Also store the initial value for auxiliary trace generation
        if boundary == Boundary::First {
            // TODO: May be invalid? For now, we put its value to zero
            if let BusBoundary::PublicInputTable(_) = bus_boundary {
                let value = ir
                    .constraint_graph_mut()
                    .insert_node(Operation::Value(crate::Value::Constant(0)));
                ir.buses_initial_values.insert(bus_index, value);
            } else {
                ir.buses_initial_values.insert(bus_index, value);
            }
        }
    }

    /// Helper function to expand the integrity constraint of a multiset bus
    fn expand_multiset_constraint(
        &self,
        ir: &mut Air,
        bus_ops: Vec<BusOp>,
        bus_access: NodeIndex,
        bus_access_with_offset: NodeIndex,
        bus_index: usize,
        tag: Option<u64>,
        constraint_form: BusConstraintForm,
    ) {
        if constraint_form == BusConstraintForm::Sum {
            self.expand_multiset_constraint_sum_form(
                ir,
                bus_ops,
                bus_access,
                bus_access_with_offset,
                bus_index,
                tag,
            );
            return;
        }
        let graph = ir.constraint_graph_mut();

        let mut p_factor = None;
        let mut p_prime_factor = None;

        for bus_op in bus_ops {
            // Expand bus operations
            let columns = bus_op.columns.clone(); // columns are the bus_operations (insert or remove of a Vec of arguments)
            let latch = bus_op.latch; // latch is the selector
            let bus_op_kind = bus_op.op_kind; // kind is either insert or remove

            // Example:
            // p.insert(a, b) when s
            // p.remove(c, d) when (1 - s)
            // => p' * (( A0 + A1 c + A2 d ) ( 1 - s ) + s) = p * ( A0 + A1 a + A2 b ) s + 1 - s

            // p' * ( columns removed combined with alphas ) = p * ( columns inserted combined with
            // alphas )
            let mut args_combined =
                graph.insert_node(Operation::Value(crate::Value::RandomValue(0)));

            for (col_index, column) in columns.iter().enumerate() {
                // 1. Combine args with alphas
                // 1.1 Start with the first alpha

                // 1.2 Create corresponding alpha
                let alpha =
                    graph.insert_node(Operation::Value(crate::Value::RandomValue(col_index + 1)));

                // 1.3 Multiply arg with alpha
                let arg_times_alpha = graph.insert_node(Operation::Mul(*column, alpha));

                // 1.4 Combine with other args
                args_combined = graph.insert_node(Operation::Add(args_combined, arg_times_alpha));
            }

            // 2. Multiply by latch
            let args_combined_with_latch = graph.insert_node(Operation::Mul(args_combined, latch));

            // 3. add inverse of latch
            let one = graph.insert_node(Operation::Value(crate::Value::Constant(1)));
            let inverse_latch = graph.insert_node(Operation::Sub(one, latch));
            let args_combined_with_latch_and_latch_inverse =
                graph.insert_node(Operation::Add(args_combined_with_latch, inverse_latch));

            // 4. Multiply them to p_factor or p_prime_factor (depending on bus_op_kind: insert: p,
            //    remove: p_prime)
            match bus_op_kind {
                BusOpKind::Insert => {
                    p_factor = match p_factor {
                        Some(p_factor) => Some(graph.insert_node(Operation::Mul(
                            p_factor,
                            args_combined_with_latch_and_latch_inverse,
                        ))),
                        None => Some(args_combined_with_latch_and_latch_inverse),
                    };
                },
                BusOpKind::Remove => {
                    p_prime_factor = match p_prime_factor {
                        Some(p_prime_factor) => Some(graph.insert_node(Operation::Mul(
                            p_prime_factor,
                            args_combined_with_latch_and_latch_inverse,
                        ))),
                        None => Some(args_combined_with_latch_and_latch_inverse),
                    };
                },
            }
        }

        // 5. Multiply the factors with the bus column (with and without offset for p' and p
        //    respectively)
        let p_prod = match p_factor {
            Some(p_factor) => graph.insert_node(Operation::Mul(p_factor, bus_access)),
            None => bus_access,
        };
        let p_prime_prod = match p_prime_factor {
            Some(p_prime_factor) => {
                graph.insert_node(Operation::Mul(p_prime_factor, bus_access_with_offset))
            },
            None => bus_access_with_offset,
        };

        // 6. Create the resulting constraint and insert it into the graph
        let root = graph.insert_node(Operation::Sub(p_prod, p_prime_prod));

        ir.constraints.insert_constraint(
            TraceSegmentId::Aux,
            root,
            ConstraintDomain::EveryFrame(2),
            tag,
        );

        // Also store the expression to computed p_prime for auxiliary trace generation
        ir.buses_transitions.insert(bus_index, (p_prod, p_prime_factor));
    }

    /// Helper function to expand multiset constraints using the sum form
    /// (assumes mutually-exclusive latches).
    fn expand_multiset_constraint_sum_form(
        &self,
        ir: &mut Air,
        bus_ops: Vec<BusOp>,
        bus_access: NodeIndex,
        bus_access_with_offset: NodeIndex,
        bus_index: usize,
        tag: Option<u64>,
    ) {
        let graph = ir.constraint_graph_mut();

        let one = graph.insert_node(Operation::Value(crate::Value::Constant(1)));
        let zero = graph.insert_node(Operation::Value(crate::Value::Constant(0)));

        let mut insert_sum = zero;
        let mut insert_flag_sum = zero;
        let mut remove_sum = zero;
        let mut remove_flag_sum = zero;

        let mut insert_groups: BTreeMap<NodeIndex, NodeIndex> = BTreeMap::new();
        let mut remove_groups: BTreeMap<NodeIndex, NodeIndex> = BTreeMap::new();

        for bus_op in bus_ops {
            let columns = bus_op.columns.clone();
            let latch = bus_op.latch;
            let bus_op_kind = bus_op.op_kind;

            // args_combined = alpha0 + alpha1*col1 + ...
            let mut args_combined =
                graph.insert_node(Operation::Value(crate::Value::RandomValue(0)));
            for (col_index, column) in columns.iter().enumerate() {
                let alpha =
                    graph.insert_node(Operation::Value(crate::Value::RandomValue(col_index + 1)));
                let arg_times_alpha = graph.insert_node(Operation::Mul(*column, alpha));
                args_combined = graph.insert_node(Operation::Add(args_combined, arg_times_alpha));
            }

            let groups = match bus_op_kind {
                BusOpKind::Insert => &mut insert_groups,
                BusOpKind::Remove => &mut remove_groups,
            };

            groups
                .entry(latch)
                .and_modify(|existing| {
                    let prod = graph.insert_node(Operation::Mul(*existing, args_combined));
                    *existing = prod;
                })
                .or_insert(args_combined);
        }

        for (latch, msg_prod) in insert_groups {
            let term = graph.insert_node(Operation::Mul(msg_prod, latch));
            insert_sum = graph.insert_node(Operation::Add(insert_sum, term));
            insert_flag_sum = graph.insert_node(Operation::Add(insert_flag_sum, latch));
        }

        for (latch, msg_prod) in remove_groups {
            let term = graph.insert_node(Operation::Mul(msg_prod, latch));
            remove_sum = graph.insert_node(Operation::Add(remove_sum, term));
            remove_flag_sum = graph.insert_node(Operation::Add(remove_flag_sum, latch));
        }

        // response = sum(insert_i * v_i) + (1 - sum(insert_i))
        let insert_rest = graph.insert_node(Operation::Sub(one, insert_flag_sum));
        let insert_factor = graph.insert_node(Operation::Add(insert_sum, insert_rest));
        // request = sum(remove_i * v_i) + (1 - sum(remove_i))
        let remove_rest = graph.insert_node(Operation::Sub(one, remove_flag_sum));
        let remove_factor = graph.insert_node(Operation::Add(remove_sum, remove_rest));

        let p_prod = graph.insert_node(Operation::Mul(insert_factor, bus_access));
        let p_prime_prod = graph.insert_node(Operation::Mul(remove_factor, bus_access_with_offset));
        // Match Miden VM ordering: s_aux' * request - s_aux * response.
        let root = graph.insert_node(Operation::Sub(p_prime_prod, p_prod));

        ir.constraints.insert_constraint(
            TraceSegmentId::Aux,
            root,
            ConstraintDomain::EveryFrame(2),
            tag,
        );

        // Also store the expression to compute p' for auxiliary trace generation.
        ir.buses_transitions.insert(bus_index, (p_prod, Some(remove_factor)));
    }

    /// Helper function to expand the integrity constraint of a logup bus
    fn expand_logup_constraint(
        &self,
        ir: &mut Air,
        bus_ops: Vec<BusOp>,
        bus_access: NodeIndex,
        bus_access_with_offset: NodeIndex,
        bus_index: usize,
        tag: Option<u64>,
    ) {
        let graph = ir.constraint_graph_mut();
        // Example:
        // q.insert(a, b, c) with d
        // q.remove(e, f, g) when s
        // => q' + s / ( A0 + A1 e + A2 f + A3 g ) = q + d / ( A0 + A1 a + A2 b + A3 c )

        //  q' + s / ( columns removed combined with alphas ) = q + d / ( columns inserted combined
        // with alphas ) PROD * q' + s * ( columns inserted combined with alphas ) = PROD *
        // q + d * ( columns removed combined with alphas )

        // 1. Compute all the factors

        let mut factors = vec![];
        for bus_op in bus_ops.iter() {
            // Expand bus operations
            let columns = bus_op.columns.clone(); // columns are the bus_operations (insert or remove of a Vec of arguments)

            // 1. Combine args with alphas
            // 1.1 Start with the first alpha
            let mut args_combined =
                graph.insert_node(Operation::Value(crate::Value::RandomValue(0)));

            for (col_index, column) in columns.iter().enumerate() {
                // 1.2 Create corresponding alpha
                let alpha =
                    graph.insert_node(Operation::Value(crate::Value::RandomValue(col_index + 1)));

                // 1.3 Multiply arg with alpha
                let arg_times_alpha = graph.insert_node(Operation::Mul(*column, alpha));

                // 1.4 Combine with other args
                args_combined = graph.insert_node(Operation::Add(args_combined, arg_times_alpha));
            }
            factors.push(args_combined);
        }

        // 2. Group identical denominators (factors) and sum their latches.
        //
        // If the same denominator appears multiple times (e.g., the same value is inserted in
        // multiple bus ops), we must not multiply it repeatedly in the common denominator.
        // The LogUp identity is:
        //
        //   q' + Σ(remove_i / den_i) = q + Σ(insert_i / den_i)
        //
        // Grouping identical den_i gives:
        //
        //   q' + Σ( (sum_remove_j) / den_j ) = q + Σ( (sum_insert_j) / den_j )
        //
        // which corresponds to using a single copy of each unique denominator. This keeps the
        // bus abstraction (send/receive) while matching constraints that are written directly
        // with a shared denominator.
        #[derive(Default)]
        struct FactorGroup {
            insert_sum: Option<NodeIndex>,
            remove_sum: Option<NodeIndex>,
        }

        let mut add_to_sum = |acc: &mut Option<NodeIndex>, value: NodeIndex| {
            *acc = Some(match *acc {
                Some(existing) => graph.insert_node(Operation::Add(existing, value)),
                None => value,
            });
        };

        let mut groups: BTreeMap<NodeIndex, FactorGroup> = BTreeMap::new();
        for (bus_index, bus_op) in bus_ops.iter().enumerate() {
            let factor = factors[bus_index];
            let entry = groups.entry(factor).or_default();
            match bus_op.op_kind {
                BusOpKind::Insert => add_to_sum(&mut entry.insert_sum, bus_op.latch),
                BusOpKind::Remove => add_to_sum(&mut entry.remove_sum, bus_op.latch),
            };
        }

        let unique_factors: Vec<(NodeIndex, FactorGroup)> = groups.into_iter().collect();

        // 3. Compute the product of all *unique* factors (used to multiply q and q').
        let mut total_factors = None;
        for (factor, _) in unique_factors.iter() {
            total_factors = match total_factors {
                Some(total_factors) => {
                    Some(graph.insert_node(Operation::Mul(total_factors, *factor)))
                },
                None => Some(*factor),
            };
        }

        // 4. For each unique denominator, compute D / den and multiply by the summed latch.
        let mut terms_added_to_bus = None;
        let mut terms_removed_from_bus = None;

        for (idx, (_factor, group)) in unique_factors.iter().enumerate() {
            // Compute product of all unique factors except the current one.
            let mut factors_without_current = None;
            for (j, (other_factor, _)) in unique_factors.iter().enumerate() {
                if j != idx {
                    factors_without_current = match factors_without_current {
                        Some(prod) => Some(graph.insert_node(Operation::Mul(prod, *other_factor))),
                        None => Some(*other_factor),
                    };
                }
            }

            // term = sum_latch * (D / den). If D/den is 1, this is just sum_latch.
            if let Some(insert_sum) = group.insert_sum {
                let term = match factors_without_current {
                    Some(scale) => graph.insert_node(Operation::Mul(insert_sum, scale)),
                    None => insert_sum,
                };
                terms_added_to_bus = match terms_added_to_bus {
                    Some(acc) => Some(graph.insert_node(Operation::Add(acc, term))),
                    None => Some(term),
                };
            }

            if let Some(remove_sum) = group.remove_sum {
                let term = match factors_without_current {
                    Some(scale) => graph.insert_node(Operation::Mul(remove_sum, scale)),
                    None => remove_sum,
                };
                terms_removed_from_bus = match terms_removed_from_bus {
                    Some(acc) => Some(graph.insert_node(Operation::Add(acc, term))),
                    None => Some(term),
                };
            }
        }

        // 4. Add all the terms together
        let q_prod = match total_factors {
            Some(total_factors) => graph.insert_node(Operation::Mul(total_factors, bus_access)),
            None => bus_access,
        };
        let q_prime_prod = match total_factors {
            Some(total_factors) => {
                graph.insert_node(Operation::Mul(total_factors, bus_access_with_offset))
            },
            None => bus_access_with_offset,
        };
        let q_term = match terms_added_to_bus {
            Some(terms_added_to_bus) => {
                graph.insert_node(Operation::Add(q_prod, terms_added_to_bus))
            },
            None => q_prod,
        };
        let q_prime_term = match terms_removed_from_bus {
            Some(terms_removed_from_bus) => {
                graph.insert_node(Operation::Add(q_prime_prod, terms_removed_from_bus))
            },
            None => q_prime_prod,
        };

        // 5. Create the resulting constraint
        let root = graph.insert_node(Operation::Sub(q_prime_term, q_term));

        // Also store the expression to computed q_prime for auxiliary trace generation
        // Note: TODO: Potentially adapt CSE to handle this properly, otherwise indices might
        // change...
        let numerator = match terms_removed_from_bus {
            Some(terms_removed_from_bus) => {
                graph.insert_node(Operation::Sub(q_term, terms_removed_from_bus))
            },
            None => q_term,
        };

        ir.constraints.insert_constraint(
            TraceSegmentId::Aux,
            root,
            ConstraintDomain::EveryFrame(2),
            tag,
        );

        ir.buses_transitions.insert(bus_index, (numerator, total_factors));
    }
}
