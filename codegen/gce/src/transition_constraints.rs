use crate::helpers::get_random_value_index;

use super::error::ConstraintEvaluationError;
use super::helpers::{
    get_constant_index_by_matrix_access, get_constant_index_by_name, get_constant_index_by_value,
    get_constant_index_by_vector_access, Expression, NodeReference, NodeType,
};
use ir::{
    transition_stmts::{ConstantValue, Operation},
    AirIR, NodeIndex,
};
use std::collections::BTreeMap;

const MAIN_TRACE_SEGMENT_INDEX: u8 = 0;

pub fn set_transition_expressions(
    ir: &AirIR,
    expressions: &mut Vec<Expression>,
    constants: &[u64],
    expressions_map: &mut BTreeMap<usize, usize>,
) -> Result<(), ConstraintEvaluationError> {
    // TODO: currently we can't create a node reference to the last row (which is required for
    // main.last and aux.last boundary constraints). Working in assumption that first reference to
    // the column is .first constraint and second is .last constraint (in the boundary section, not
    // entire array)
    for (index, node) in ir.transition_graph().nodes().iter().enumerate() {
        match node.op() {
            Operation::Add(l, r) => {
                expressions.push(handle_transition_expression(
                    ir,
                    "ADD".to_string(),
                    *l,
                    *r,
                    constants,
                    expressions_map,
                )?);
                expressions_map.insert(index, expressions.len() - 1);
            }
            Operation::Sub(l, r) => {
                expressions.push(handle_transition_expression(
                    ir,
                    "SUB".to_string(),
                    *l,
                    *r,
                    constants,
                    expressions_map,
                )?);
                expressions_map.insert(index, expressions.len() - 1);
            }
            Operation::Mul(l, r) => {
                expressions.push(handle_transition_expression(
                    ir,
                    "MUL".to_string(),
                    *l,
                    *r,
                    constants,
                    expressions_map,
                )?);
                expressions_map.insert(index, expressions.len() - 1);
            }
            Operation::Exp(i, degree) => {
                match degree {
                    0 => {
                        // I decided that node^0 could be emulated using the product of 1*1, but perhaps there are better ways
                        let index_of_1 = get_constant_index_by_value(1, constants)?;
                        let const_1_node = NodeReference {
                            node_type: NodeType::CONST,
                            index: index_of_1,
                        };
                        expressions.push(Expression {
                            op: "MUL".to_string(),
                            lhs: const_1_node.clone(),
                            rhs: const_1_node,
                        });
                    }
                    1 => {
                        let lhs = handle_node_reference(ir, *i, constants, expressions_map)?;
                        let degree_index = get_constant_index_by_value(1, constants)?;
                        let rhs = NodeReference {
                            node_type: NodeType::CONST,
                            index: degree_index,
                        };
                        expressions.push(Expression {
                            op: "MUL".to_string(),
                            lhs,
                            rhs,
                        });
                    }
                    _ => handle_exponentiation(
                        ir,
                        expressions,
                        expressions_map,
                        *i,
                        *degree,
                        constants,
                    )?,
                }
                expressions_map.insert(index, expressions.len() - 1);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Fills the `outputs` vector according to the indexes from `expressions_map`
pub fn set_transition_outputs(
    ir: &AirIR,
    outputs: &mut Vec<usize>,
    expressions_map: &BTreeMap<usize, usize>,
) -> Result<(), ConstraintEvaluationError> {
    for i in 0..ir.num_polys().len() {
        for root in ir.transition_constraints(i as u8) {
            let index = expressions_map
                .get(&root.index())
                .ok_or_else(|| ConstraintEvaluationError::operation_not_found(root.index()))?;
            outputs.push(*index);
        }
    }
    Ok(())
}

/// Parses expression in transition graph Node vector and returns related [Expression] instance
fn handle_transition_expression(
    ir: &AirIR,
    op: String,
    l: NodeIndex,
    r: NodeIndex,
    constants: &[u64],
    expressions_map: &BTreeMap<usize, usize>,
) -> Result<Expression, ConstraintEvaluationError> {
    let lhs = handle_node_reference(ir, l, constants, expressions_map)?;
    let rhs = handle_node_reference(ir, r, constants, expressions_map)?;
    Ok(Expression { op, lhs, rhs })
}

/// Parses expression in transition graph Node vector by [NodeIndex] and returns related
/// [NodeReference] instance
fn handle_node_reference(
    ir: &AirIR,
    i: NodeIndex,
    constants: &[u64],
    expressions_map: &BTreeMap<usize, usize>,
) -> Result<NodeReference, ConstraintEvaluationError> {
    use Operation::*;
    match ir.transition_graph().node(&i).op() {
        Add(_, _) | Sub(_, _) | Mul(_, _) | Exp(_, _) => {
            let index = expressions_map
                .get(&i.index())
                .ok_or_else(|| ConstraintEvaluationError::operation_not_found(i.index()))?;
            Ok(NodeReference {
                node_type: NodeType::EXPR,
                index: *index,
            })
        }
        Constant(constant_value) => {
            match constant_value {
                ConstantValue::Inline(v) => {
                    let index = get_constant_index_by_value(*v, constants)?;
                    Ok(NodeReference {
                        node_type: NodeType::CONST,
                        index,
                    })
                }
                ConstantValue::Scalar(name) => {
                    let index = get_constant_index_by_name(ir, name, constants)?;
                    Ok(NodeReference {
                        node_type: NodeType::CONST,
                        index,
                    })
                }
                ConstantValue::Vector(vector_access) => {
                    // why Constant.name() returns Identifier and VectorAccess.name() works like
                    // VectorAccess.name.name() and returns &str? (same with MatrixAccess)
                    let index = get_constant_index_by_vector_access(ir, vector_access, constants)?;
                    Ok(NodeReference {
                        node_type: NodeType::CONST,
                        index,
                    })
                }
                ConstantValue::Matrix(matrix_access) => {
                    let index = get_constant_index_by_matrix_access(ir, matrix_access, constants)?;
                    Ok(NodeReference {
                        node_type: NodeType::CONST,
                        index,
                    })
                }
            }
        }
        TraceElement(trace_access) => {
            // Working in assumption that segment 0 is main columns, and others are main columns
            match trace_access.trace_segment() {
                MAIN_TRACE_SEGMENT_INDEX => {
                    // TODO: handle other offsets (not only 1)
                    if trace_access.row_offset() == 0 {
                        Ok(NodeReference {
                            node_type: NodeType::POL,
                            index: trace_access.col_idx(),
                        })
                    } else {
                        Ok(NodeReference {
                            node_type: NodeType::POL_NEXT,
                            index: trace_access.col_idx(),
                        })
                    }
                }
                i if i < ir.num_polys().len() as u8 => {
                    let col_index = ir.num_polys()[0..i as usize].iter().sum::<u16>() as usize
                        + trace_access.col_idx();
                    if trace_access.row_offset() == 0 {
                        Ok(NodeReference {
                            node_type: NodeType::POL,
                            index: col_index,
                        })
                    } else {
                        Ok(NodeReference {
                            node_type: NodeType::POL_NEXT,
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
            let index = get_random_value_index(ir, *rand_index);
            Ok(NodeReference {
                node_type: NodeType::VAR,
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
/// expressions vector
fn handle_exponentiation(
    ir: &AirIR,
    expressions: &mut Vec<Expression>,
    expressions_map: &BTreeMap<usize, usize>,
    i: NodeIndex,
    degree: usize,
    constants: &[u64],
) -> Result<(), ConstraintEvaluationError> {
    // base node that we want to raise to a degree
    let base_node = handle_node_reference(ir, i, constants, expressions_map)?;
    // push node^2 expression
    expressions.push(Expression {
        op: "MUL".to_string(),
        lhs: base_node.clone(),
        rhs: base_node.clone(),
    });
    let square_node_index = expressions.len() - 1;

    // square the previous expression while there is such an opportunity
    let mut cur_degree_of_2 = 1; // currently we have node^(2^cur_degree_of_2) = node^(2^1) = node^2
    while 2_usize.pow(cur_degree_of_2) <= degree / 2 {
        let last_node = NodeReference {
            node_type: NodeType::EXPR,
            index: expressions.len() - 1,
        };
        expressions.push(Expression {
            op: "MUL".to_string(),
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
                node_type: NodeType::EXPR,
                index: expressions.len() - 1,
            };
            expressions.push(Expression {
                op: "MUL".to_string(),
                lhs: last_node,
                rhs: base_node,
            });
            break;
        }
        if 2_usize.pow(cur_degree_of_2 - 1) <= diff {
            let last_node = NodeReference {
                node_type: NodeType::EXPR,
                index: expressions.len() - 1,
            };
            let fitting_degree_of_2_node = NodeReference {
                node_type: NodeType::EXPR,
                // cur_degree_of_2 shows how many indexes we need to add to reach the largest fitting degree of 2
                index: square_node_index + cur_degree_of_2 as usize - 2,
            };
            expressions.push(Expression {
                op: "MUL".to_string(),
                lhs: last_node,
                rhs: fitting_degree_of_2_node,
            });
            cur_max_degree += 2_usize.pow(cur_degree_of_2 - 1);
        }
        cur_degree_of_2 -= 1;
    }

    Ok(())
}
