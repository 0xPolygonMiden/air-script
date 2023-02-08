use super::error::ConstraintEvaluationError;
use super::{
    utils::{
        get_constant_index_by_matrix_access, get_constant_index_by_name,
        get_constant_index_by_value, get_constant_index_by_vector_access, get_random_value_index,
    },
    ExpressionJson, ExpressionOperation, NodeReference, NodeType,
};
use ir::{
    constraints::{ConstantValue, Operation},
    AirIR, NodeIndex,
};
use std::collections::BTreeMap;

const MAIN_TRACE_SEGMENT_INDEX: u8 = 0;

pub struct ExpressionsHandler<'a> {
    ir: &'a AirIR,
    constants: &'a [u64],
    // maps indexes in Node vector in AlgebraicGraph and in `expressions` JSON array
    expressions_map: &'a mut BTreeMap<usize, usize>,
}

impl<'a> ExpressionsHandler<'a> {
    pub fn new(
        ir: &'a AirIR,
        constants: &'a [u64],
        expressions_map: &'a mut BTreeMap<usize, usize>,
    ) -> Self {
        ExpressionsHandler {
            ir,
            constants,
            expressions_map,
        }
    }

    /// Parses expressions in transition graph's Node vector, creates [Expression] instances and pushes
    /// them to the `expressions` vector.
    pub fn get_expressions(&mut self) -> Result<Vec<ExpressionJson>, ConstraintEvaluationError> {
        // TODO: currently we can't create a node reference to the last row (which is required for
        // main.last and aux.last boundary constraints). Working in assumption that first reference to
        // the column is .first constraint and second is .last constraint (in the boundary section, not
        // entire array)
        let mut expressions = Vec::new();

        for (index, node) in self.ir.constraint_graph().nodes().iter().enumerate() {
            match node.op() {
                Operation::Add(l, r) => {
                    expressions.push(self.handle_transition_expression(
                        ExpressionOperation::Add,
                        *l,
                        *r,
                    )?);
                    // create mapping (index in node graph: index in expressions vector)
                    self.expressions_map.insert(index, expressions.len() - 1);
                }
                Operation::Sub(l, r) => {
                    expressions.push(self.handle_transition_expression(
                        ExpressionOperation::Sub,
                        *l,
                        *r,
                    )?);
                    self.expressions_map.insert(index, expressions.len() - 1);
                }
                Operation::Mul(l, r) => {
                    expressions.push(self.handle_transition_expression(
                        ExpressionOperation::Mul,
                        *l,
                        *r,
                    )?);
                    self.expressions_map.insert(index, expressions.len() - 1);
                }
                Operation::Exp(i, degree) => {
                    match degree {
                        0 => {
                            // I decided that node^0 could be emulated using the product of 1*1, but perhaps there are better ways
                            let index_of_1 = get_constant_index_by_value(1, self.constants)?;
                            let const_1_node = NodeReference {
                                node_type: NodeType::Const,
                                index: index_of_1,
                            };
                            expressions.push(ExpressionJson {
                                op: ExpressionOperation::Mul,
                                lhs: const_1_node.clone(),
                                rhs: const_1_node,
                            });
                        }
                        1 => {
                            let lhs = self.handle_node_reference(*i)?;
                            let degree_index = get_constant_index_by_value(1, self.constants)?;
                            let rhs = NodeReference {
                                node_type: NodeType::Const,
                                index: degree_index,
                            };
                            expressions.push(ExpressionJson {
                                op: ExpressionOperation::Mul,
                                lhs,
                                rhs,
                            });
                        }
                        _ => self.handle_exponentiation(&mut expressions, *i, *degree)?,
                    }
                    self.expressions_map.insert(index, expressions.len() - 1);
                }
                _ => {}
            }
        }
        Ok(expressions)
    }

    /// Fills the `outputs` vector with indexes from `expressions` vector according to the `expressions_map`.
    pub fn get_outputs(
        &self,
        expressions: &mut Vec<ExpressionJson>,
    ) -> Result<Vec<usize>, ConstraintEvaluationError> {
        let mut outputs = Vec::new();

        for i in 0..self.ir.segment_widths().len() {
            for root in self.ir.boundary_constraints(i as u8) {
                let index = self
                    .expressions_map
                    .get(&root.node_index().index())
                    .ok_or_else(|| {
                        ConstraintEvaluationError::operation_not_found(root.node_index().index())
                    })?;
                if outputs.contains(index) {
                    expressions.push(expressions[*index].clone());
                    outputs.push(expressions.len() - 1);
                } else {
                    outputs.push(*index);
                }
            }

            for root in self.ir.validity_constraints(i as u8) {
                let index = self
                    .expressions_map
                    .get(&root.node_index().index())
                    .ok_or_else(|| {
                        ConstraintEvaluationError::operation_not_found(root.node_index().index())
                    })?;
                outputs.push(*index);
            }

            for root in self.ir.transition_constraints(i as u8) {
                let index = self
                    .expressions_map
                    .get(&root.node_index().index())
                    .ok_or_else(|| {
                        ConstraintEvaluationError::operation_not_found(root.node_index().index())
                    })?;
                outputs.push(*index);
            }
        }
        Ok(outputs)
    }

    // --- HELPERS --------------------------------------------------------------------------------

    /// Parses expression in transition graph Node vector and returns related [Expression] instance.
    fn handle_transition_expression(
        &self,
        op: ExpressionOperation,
        l: NodeIndex,
        r: NodeIndex,
    ) -> Result<ExpressionJson, ConstraintEvaluationError> {
        let lhs = self.handle_node_reference(l)?;
        let rhs = self.handle_node_reference(r)?;
        Ok(ExpressionJson { op, lhs, rhs })
    }

    /// Parses expression in transition graph Node vector by [NodeIndex] and returns related
    /// [NodeReference] instance.
    fn handle_node_reference(
        &self,
        i: NodeIndex,
    ) -> Result<NodeReference, ConstraintEvaluationError> {
        use Operation::*;
        match self.ir.constraint_graph().node(&i).op() {
            Add(_, _) | Sub(_, _) | Mul(_, _) | Exp(_, _) => {
                let index = self
                    .expressions_map
                    .get(&i.index())
                    .ok_or_else(|| ConstraintEvaluationError::operation_not_found(i.index()))?;
                Ok(NodeReference {
                    node_type: NodeType::Expr,
                    index: *index,
                })
            }
            Constant(constant_value) => {
                match constant_value {
                    ConstantValue::Inline(v) => {
                        let index = get_constant_index_by_value(*v, self.constants)?;
                        Ok(NodeReference {
                            node_type: NodeType::Const,
                            index,
                        })
                    }
                    ConstantValue::Scalar(name) => {
                        let index = get_constant_index_by_name(self.ir, name, self.constants)?;
                        Ok(NodeReference {
                            node_type: NodeType::Const,
                            index,
                        })
                    }
                    ConstantValue::Vector(vector_access) => {
                        // why Constant.name() returns Identifier and VectorAccess.name() works like
                        // VectorAccess.name.name() and returns &str? (same with MatrixAccess)
                        let index = get_constant_index_by_vector_access(
                            self.ir,
                            vector_access,
                            self.constants,
                        )?;
                        Ok(NodeReference {
                            node_type: NodeType::Const,
                            index,
                        })
                    }
                    ConstantValue::Matrix(matrix_access) => {
                        let index = get_constant_index_by_matrix_access(
                            self.ir,
                            matrix_access,
                            self.constants,
                        )?;
                        Ok(NodeReference {
                            node_type: NodeType::Const,
                            index,
                        })
                    }
                }
            }
            TraceElement(trace_access) => {
                // Working in assumption that segment 0 is main columns, and others are aux columns
                match trace_access.trace_segment() {
                    MAIN_TRACE_SEGMENT_INDEX => {
                        // TODO: handle other offsets (not only 1)
                        if trace_access.row_offset() == 0 {
                            Ok(NodeReference {
                                node_type: NodeType::Pol,
                                index: trace_access.col_idx(),
                            })
                        } else {
                            Ok(NodeReference {
                                node_type: NodeType::PolNext,
                                index: trace_access.col_idx(),
                            })
                        }
                    }
                    i if i < self.ir.segment_widths().len() as u8 => {
                        let col_index = self.ir.segment_widths()[0..i as usize].iter().sum::<u16>()
                            as usize
                            + trace_access.col_idx();
                        if trace_access.row_offset() == 0 {
                            Ok(NodeReference {
                                node_type: NodeType::Pol,
                                index: col_index,
                            })
                        } else {
                            Ok(NodeReference {
                                node_type: NodeType::PolNext,
                                index: col_index,
                            })
                        }
                    }
                    _ => Err(ConstraintEvaluationError::invalid_trace_segment(
                        trace_access.trace_segment(),
                    )),
                }
            }
            RandomValue(rand_index) => {
                let index = get_random_value_index(self.ir, *rand_index);
                Ok(NodeReference {
                    node_type: NodeType::Var,
                    index,
                })
            }

            PeriodicColumn(_column, _length) => todo!(),

            // Currently it can only be `Neg`
            _ => Err(ConstraintEvaluationError::InvalidOperation(
                "Invalid transition constraint operation".to_string(),
            )),
        }
    }

    /// Replaces the exponentiation operation with multiplication operations, adding them to the
    /// expressions vector.
    fn handle_exponentiation(
        &self,
        expressions: &mut Vec<ExpressionJson>,
        i: NodeIndex,
        degree: usize,
    ) -> Result<(), ConstraintEvaluationError> {
        // base node that we want to raise to a degree
        let base_node = self.handle_node_reference(i)?;
        // push node^2 expression
        expressions.push(ExpressionJson {
            op: ExpressionOperation::Mul,
            lhs: base_node.clone(),
            rhs: base_node.clone(),
        });
        let square_node_index = expressions.len() - 1;

        // square the previous expression while there is such an opportunity
        let mut cur_degree_of_2 = 1; // currently we have node^(2^cur_degree_of_2) = node^(2^1) = node^2
        while 2_usize.pow(cur_degree_of_2) <= degree / 2 {
            // the last node that we want to square
            let last_node = NodeReference {
                node_type: NodeType::Expr,
                index: expressions.len() - 1,
            };
            expressions.push(ExpressionJson {
                op: ExpressionOperation::Mul,
                lhs: last_node.clone(),
                rhs: last_node,
            });
            cur_degree_of_2 += 1;
        }

        // add the largest available powers of two to the current degree
        let mut cur_max_degree = 2_usize.pow(cur_degree_of_2); // currently we have node^(2^cur_max_degree)
        while cur_max_degree != degree {
            let diff = degree - cur_max_degree;
            if diff == 1 {
                // if we need to add first degree (base node)
                let last_node = NodeReference {
                    node_type: NodeType::Expr,
                    index: expressions.len() - 1,
                };
                expressions.push(ExpressionJson {
                    op: ExpressionOperation::Mul,
                    lhs: last_node,
                    rhs: base_node,
                });
                break;
            }
            if 2_usize.pow(cur_degree_of_2 - 1) <= diff {
                let last_node = NodeReference {
                    node_type: NodeType::Expr,
                    index: expressions.len() - 1,
                };
                let fitting_degree_of_2_node = NodeReference {
                    node_type: NodeType::Expr,
                    // cur_degree_of_2 shows how many indexes we need to add to reach the largest fitting degree of 2
                    index: square_node_index + cur_degree_of_2 as usize - 2,
                };
                expressions.push(ExpressionJson {
                    op: ExpressionOperation::Mul,
                    lhs: last_node,
                    rhs: fitting_degree_of_2_node,
                });
                cur_max_degree += 2_usize.pow(cur_degree_of_2 - 1);
            }
            cur_degree_of_2 -= 1;
        }

        Ok(())
    }
}
