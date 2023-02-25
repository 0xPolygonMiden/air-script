use super::error::ConstraintEvaluationError;
use super::{
    utils::{
        get_constant_index_by_matrix_access, get_constant_index_by_name,
        get_constant_index_by_value, get_constant_index_by_vector_access, get_public_input_index,
        get_random_value_index,
    },
    ExpressionJson, ExpressionOperation, NodeReference, NodeType,
};
use ir::{
    constraints::{ConstantValue, Operation},
    AirIR, NodeIndex,
};
use std::collections::BTreeMap;

const MAIN_TRACE_SEGMENT_INDEX: u8 = 0;

pub struct GceBuilder {
    // maps indexes in Node vector in AlgebraicGraph and in `expressions` JSON array
    expressions_map: BTreeMap<usize, usize>,
    expressions: Vec<ExpressionJson>,
    outputs: Vec<usize>,
}

impl GceBuilder {
    pub fn new() -> Self {
        GceBuilder {
            expressions_map: BTreeMap::new(),
            expressions: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn build(
        &mut self,
        ir: &AirIR,
        constants: &[u64],
    ) -> Result<(), ConstraintEvaluationError> {
        self.build_expressions(ir, constants)?;
        self.build_outputs(ir)?;
        Ok(())
    }

    pub fn into_gce(self) -> Result<(Vec<ExpressionJson>, Vec<usize>), ConstraintEvaluationError> {
        Ok((self.expressions, self.outputs))
    }

    /// Parses expressions in transition graph's Node vector, creates [Expression] instances and pushes
    /// them to the `expressions` vector.
    fn build_expressions(
        &mut self,
        ir: &AirIR,
        constants: &[u64],
    ) -> Result<(), ConstraintEvaluationError> {
        // TODO: currently we can't create a node reference to the last row (which is required for
        // main.last and aux.last boundary constraints). Working in assumption that first reference to
        // the column is .first constraint and second is .last constraint (in the boundary section, not
        // entire array)
        for (index, node) in ir.constraint_graph().nodes().iter().enumerate() {
            match node.op() {
                Operation::Add(l, r) => {
                    self.expressions.push(self.handle_transition_expression(
                        ir,
                        constants,
                        ExpressionOperation::Add,
                        *l,
                        *r,
                    )?);
                    // create mapping (index in node graph: index in expressions vector)
                    self.expressions_map
                        .insert(index, self.expressions.len() - 1);
                }
                Operation::Sub(l, r) => {
                    self.expressions.push(self.handle_transition_expression(
                        ir,
                        constants,
                        ExpressionOperation::Sub,
                        *l,
                        *r,
                    )?);
                    self.expressions_map
                        .insert(index, self.expressions.len() - 1);
                }
                Operation::Mul(l, r) => {
                    self.expressions.push(self.handle_transition_expression(
                        ir,
                        constants,
                        ExpressionOperation::Mul,
                        *l,
                        *r,
                    )?);
                    self.expressions_map
                        .insert(index, self.expressions.len() - 1);
                }
                Operation::Exp(i, degree) => {
                    match degree {
                        0 => {
                            // I decided that node^0 could be emulated using the product of 1*1, but perhaps there are better ways
                            let index_of_1 = get_constant_index_by_value(1, constants)?;
                            let const_1_node = NodeReference {
                                node_type: NodeType::Const,
                                index: index_of_1,
                            };
                            self.expressions.push(ExpressionJson {
                                op: ExpressionOperation::Mul,
                                lhs: const_1_node.clone(),
                                rhs: const_1_node,
                            });
                        }
                        1 => {
                            let lhs = self.handle_node_reference(ir, constants, *i)?;
                            let degree_index = get_constant_index_by_value(1, constants)?;
                            let rhs = NodeReference {
                                node_type: NodeType::Const,
                                index: degree_index,
                            };
                            self.expressions.push(ExpressionJson {
                                op: ExpressionOperation::Mul,
                                lhs,
                                rhs,
                            });
                        }
                        _ => self.handle_exponentiation(ir, constants, *i, *degree)?,
                    }
                    self.expressions_map
                        .insert(index, self.expressions.len() - 1);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Fills the `outputs` vector with indexes from `expressions` vector according to the `expressions_map`.
    fn build_outputs(&mut self, ir: &AirIR) -> Result<(), ConstraintEvaluationError> {
        for i in 0..ir.segment_widths().len() {
            for root in ir.boundary_constraints(i as u8) {
                let index = self
                    .expressions_map
                    .get(&root.node_index().index())
                    .ok_or_else(|| {
                        ConstraintEvaluationError::operation_not_found(root.node_index().index())
                    })?;
                // if we found index twice, put the corresponding expression in the expressions
                // array again. It means that we have equal boundary constraints for both first
                // and last domains (e.g. a.first = 1 and a.last = 1)
                if self.outputs.contains(index) {
                    self.expressions.push(self.expressions[*index].clone());
                    self.outputs.push(self.expressions.len() - 1);
                } else {
                    self.outputs.push(*index);
                }
            }

            for root in ir.validity_constraints(i as u8) {
                let index = self
                    .expressions_map
                    .get(&root.node_index().index())
                    .ok_or_else(|| {
                        ConstraintEvaluationError::operation_not_found(root.node_index().index())
                    })?;
                self.outputs.push(*index);
            }

            for root in ir.transition_constraints(i as u8) {
                let index = self
                    .expressions_map
                    .get(&root.node_index().index())
                    .ok_or_else(|| {
                        ConstraintEvaluationError::operation_not_found(root.node_index().index())
                    })?;
                self.outputs.push(*index);
            }
        }
        Ok(())
    }

    // --- HELPERS --------------------------------------------------------------------------------

    /// Parses expression in transition graph Node vector and returns related [Expression] instance.
    fn handle_transition_expression(
        &self,
        ir: &AirIR,
        constants: &[u64],
        op: ExpressionOperation,
        l: NodeIndex,
        r: NodeIndex,
    ) -> Result<ExpressionJson, ConstraintEvaluationError> {
        let lhs = self.handle_node_reference(ir, constants, l)?;
        let rhs = self.handle_node_reference(ir, constants, r)?;
        Ok(ExpressionJson { op, lhs, rhs })
    }

    /// Parses expression in transition graph Node vector by [NodeIndex] and returns related
    /// [NodeReference] instance.
    fn handle_node_reference(
        &self,
        ir: &AirIR,
        constants: &[u64],
        i: NodeIndex,
    ) -> Result<NodeReference, ConstraintEvaluationError> {
        use Operation::*;
        match ir.constraint_graph().node(&i).op() {
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
                        let index = get_constant_index_by_value(*v, constants)?;
                        Ok(NodeReference {
                            node_type: NodeType::Const,
                            index,
                        })
                    }
                    ConstantValue::Scalar(name) => {
                        let index = get_constant_index_by_name(ir, name, constants)?;
                        Ok(NodeReference {
                            node_type: NodeType::Const,
                            index,
                        })
                    }
                    ConstantValue::Vector(vector_access) => {
                        // why Constant.name() returns Identifier and VectorAccess.name() works like
                        // VectorAccess.name.name() and returns &str? (same with MatrixAccess)
                        let index =
                            get_constant_index_by_vector_access(ir, vector_access, constants)?;
                        Ok(NodeReference {
                            node_type: NodeType::Const,
                            index,
                        })
                    }
                    ConstantValue::Matrix(matrix_access) => {
                        let index =
                            get_constant_index_by_matrix_access(ir, matrix_access, constants)?;
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
                    i if i < ir.segment_widths().len() as u8 => {
                        let col_index = ir.segment_widths()[0..i as usize].iter().sum::<u16>()
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
                let index = get_random_value_index(ir, rand_index);
                Ok(NodeReference {
                    node_type: NodeType::Var,
                    index,
                })
            }
            PublicInput(name, public_index) => {
                let index = get_public_input_index(ir, name, public_index);
                Ok(NodeReference {
                    node_type: NodeType::Var,
                    index,
                })
            }

            PeriodicColumn(_column, _length) => todo!(),
        }
    }

    /// Replaces the exponentiation operation with multiplication operations, adding them to the
    /// expressions vector.
    fn handle_exponentiation(
        &mut self,
        ir: &AirIR,
        constants: &[u64],
        i: NodeIndex,
        degree: usize,
    ) -> Result<(), ConstraintEvaluationError> {
        // base node that we want to raise to a degree
        let base_node = self.handle_node_reference(ir, constants, i)?;
        // push node^2 expression
        self.expressions.push(ExpressionJson {
            op: ExpressionOperation::Mul,
            lhs: base_node.clone(),
            rhs: base_node.clone(),
        });
        let square_node_index = self.expressions.len() - 1;

        // square the previous expression while there is such an opportunity
        let mut cur_degree_of_2 = 1; // currently we have node^(2^cur_degree_of_2) = node^(2^1) = node^2
        while 2_usize.pow(cur_degree_of_2) <= degree / 2 {
            // the last node that we want to square
            let last_node = NodeReference {
                node_type: NodeType::Expr,
                index: self.expressions.len() - 1,
            };
            self.expressions.push(ExpressionJson {
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
                    index: self.expressions.len() - 1,
                };
                self.expressions.push(ExpressionJson {
                    op: ExpressionOperation::Mul,
                    lhs: last_node,
                    rhs: base_node,
                });
                break;
            }
            if 2_usize.pow(cur_degree_of_2 - 1) <= diff {
                let last_node = NodeReference {
                    node_type: NodeType::Expr,
                    index: self.expressions.len() - 1,
                };
                let fitting_degree_of_2_node = NodeReference {
                    node_type: NodeType::Expr,
                    // cur_degree_of_2 shows how many indexes we need to add to reach the largest fitting degree of 2
                    index: square_node_index + cur_degree_of_2 as usize - 2,
                };
                self.expressions.push(ExpressionJson {
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
