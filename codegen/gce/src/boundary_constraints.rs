use crate::helpers::get_random_value_index;

use super::error::ConstraintEvaluationError;
use super::helpers::{
    get_constant_index_by_matrix_access, get_constant_index_by_name, get_constant_index_by_value,
    get_constant_index_by_vector_access, get_public_input_index, Expression, NodeReference,
    NodeType,
};
use ir::{AirIR, BoundaryExpr};
use std::collections::BTreeMap;

pub fn set_boundary_expressions_and_outputs(
    ir: &AirIR,
    boundary_constraints_vec: [&Vec<(usize, &BoundaryExpr)>; 4],
    expressions: &mut Vec<Expression>,
    outputs: &mut Vec<usize>,
    constants: &Vec<u64>,
    const_public_type_map: &BTreeMap<&str, NodeType>,
) -> Result<(), ConstraintEvaluationError> {
    for (i, &constraints) in boundary_constraints_vec.iter().enumerate() {
        for constraint in constraints {
            if (0..2).contains(&i) {
                handle_boundary_operation(
                    ir,
                    constraint,
                    expressions,
                    outputs,
                    constants,
                    const_public_type_map,
                )?;
            } else {
                handle_boundary_operation(
                    ir,
                    &(constraint.0 + ir.num_polys()[0] as usize, constraint.1),
                    expressions,
                    outputs,
                    constants,
                    const_public_type_map,
                )?;
            }
        }
    }
    Ok(())
}

// Problems:
// #3: I need to be able to create NodeReferences to the last row of the table (to create reference
// to the .last boundary constraints)
// #4: Need to be able to reuse boundary expressions if it has already been created
/// Parses boundary operation, creates related [Expression] instance, pushes it to the `expressions`
/// array and adds its index to the `outputs` array
fn handle_boundary_operation(
    ir: &AirIR,
    expr: &(usize, &BoundaryExpr),
    expressions: &mut Vec<Expression>,
    outputs: &mut Vec<usize>,
    constants: &Vec<u64>,
    const_public_type_map: &BTreeMap<&str, NodeType>,
) -> Result<(), ConstraintEvaluationError> {
    use BoundaryExpr::*;
    match expr.1 {
        Const(v) => {
            let constant_index = get_constant_index_by_value(*v, constants)?;
            push_boundary_value(
                expressions,
                outputs,
                NodeType::CONST,
                constant_index,
                expr.0,
            );
            Ok(())
        }
        Elem(id) => {
            let constant_index = get_constant_index_by_name(ir, &id.0, constants)?;
            push_boundary_value(
                expressions,
                outputs,
                NodeType::CONST,
                constant_index,
                expr.0,
            );
            Ok(())
        }
        VectorAccess(vector_access) => {
            let node_type = const_public_type_map
                .get(vector_access.name())
                .ok_or_else(|| {
                    ConstraintEvaluationError::identifier_not_found(vector_access.name())
                })?;
            match node_type {
                NodeType::CONST => {
                    let constant_index =
                        get_constant_index_by_vector_access(ir, vector_access, constants)?;
                    push_boundary_value(
                        expressions,
                        outputs,
                        NodeType::CONST,
                        constant_index,
                        expr.0,
                    );
                    Ok(())
                }
                NodeType::VAR => {
                    let public_input_index = get_public_input_index(ir, vector_access)?;
                    push_boundary_value(
                        expressions,
                        outputs,
                        NodeType::VAR,
                        public_input_index,
                        expr.0,
                    );
                    Ok(())
                }
                _ => Err(ConstraintEvaluationError::InvalidOperation(
                    "Invalid node type: only CONST and VAR allowed".to_string(),
                )),
            }
        }
        MatrixAccess(matrix_access) => {
            let constant_index = get_constant_index_by_matrix_access(ir, matrix_access, constants)?;
            push_boundary_value(
                expressions,
                outputs,
                NodeType::CONST,
                constant_index,
                expr.0,
            );
            Ok(())
        }
        Rand(rand_index) => {
            let index = get_random_value_index(ir, *rand_index);
            push_boundary_value(expressions, outputs, NodeType::VAR, index, expr.0);
            Ok(())
        }
        Add(l, r) => parse_boundary_expression(
            ir,
            (expr.0, l, r),
            "ADD".to_string(),
            constants,
            const_public_type_map,
            expressions,
            outputs,
        ),
        Sub(l, r) => parse_boundary_expression(
            ir,
            (expr.0, l, r),
            "SUB".to_string(),
            constants,
            const_public_type_map,
            expressions,
            outputs,
        ),
        Mul(l, r) => parse_boundary_expression(
            ir,
            (expr.0, l, r),
            "MUL".to_string(),
            constants,
            const_public_type_map,
            expressions,
            outputs,
        ),

        Exp(_i, _degree) => todo!(),
    }
}

/// Creates an [Expression] instance on an equation of the form `boundary_constraint = expression`,
/// pushes it to the `expressions` array and adds its index to the `outputs` array
fn push_boundary_value(
    expressions: &mut Vec<Expression>,
    outputs: &mut Vec<usize>,
    node_type: NodeType,
    value_index: usize,
    column_index: usize,
) {
    let lhs = NodeReference {
        node_type: NodeType::POL,
        index: column_index,
    };
    let rhs = NodeReference {
        node_type,
        index: value_index,
    };
    let result = Expression {
        op: "SUB".to_string(),
        lhs,
        rhs,
    };
    expressions.push(result);
    outputs.push(expressions.len() - 1);
}

/// Parses boundary operation in case it is an expression. Creates [Expression], pushes it to the
/// `expressions` array and adds its index to the `outputs` array
fn parse_boundary_expression(
    ir: &AirIR,
    boundary_expr: (usize, &BoundaryExpr, &BoundaryExpr),
    op_type: String,
    constants: &Vec<u64>,
    const_public_type_map: &BTreeMap<&str, NodeType>,
    expressions: &mut Vec<Expression>,
    output: &mut Vec<usize>,
) -> Result<(), ConstraintEvaluationError> {
    let node_reference = parse_recursive_boundary_expression(
        ir,
        (boundary_expr.1, boundary_expr.2),
        op_type,
        constants,
        const_public_type_map,
        expressions,
    )?;

    let lhs = NodeReference {
        node_type: NodeType::POL,
        index: boundary_expr.0,
    };
    let rhs = node_reference;
    let result = Expression {
        op: "SUB".to_string(),
        lhs,
        rhs,
    };
    expressions.push(result);
    output.push(expressions.len() - 1);
    Ok(())
}

/// Recursively parses boundary expression.
/// Returns [NodeReference] to the parsed expression
fn parse_recursive_boundary_expression(
    ir: &AirIR,
    boundary_expr: (&BoundaryExpr, &BoundaryExpr),
    op_type: String,
    constants: &Vec<u64>,
    const_public_type_map: &BTreeMap<&str, NodeType>,
    expressions: &mut Vec<Expression>,
) -> Result<NodeReference, ConstraintEvaluationError> {
    let lhs = parse_boundary_limb(
        ir,
        boundary_expr.0,
        constants,
        const_public_type_map,
        expressions,
    )?;
    let rhs = parse_boundary_limb(
        ir,
        boundary_expr.1,
        constants,
        const_public_type_map,
        expressions,
    )?;

    let result = Expression {
        op: op_type,
        lhs,
        rhs,
    };
    expressions.push(result);

    Ok(NodeReference {
        node_type: NodeType::EXPR,
        index: expressions.len() - 1,
    })
}

/// Parses boundary expression limb.
/// Returns [NodeReference] to the parsed expression
fn parse_boundary_limb(
    ir: &AirIR,
    i: &BoundaryExpr,
    constants: &Vec<u64>,
    const_public_type_map: &BTreeMap<&str, NodeType>,
    expressions: &mut Vec<Expression>,
) -> Result<NodeReference, ConstraintEvaluationError> {
    use BoundaryExpr::*;
    match i {
        Const(v) => {
            let constant_index = get_constant_index_by_value(*v, constants)?;
            Ok(NodeReference {
                node_type: NodeType::CONST,
                index: constant_index,
            })
        }
        Elem(id) => {
            let constant_index = get_constant_index_by_name(ir, &id.0, constants)?;
            Ok(NodeReference {
                node_type: NodeType::CONST,
                index: constant_index,
            })
        }
        VectorAccess(vector_access) => {
            let node_type = const_public_type_map
                .get(vector_access.name())
                .ok_or_else(|| {
                    ConstraintEvaluationError::identifier_not_found(vector_access.name())
                })?;
            match node_type {
                NodeType::CONST => {
                    let constant_index =
                        get_constant_index_by_vector_access(ir, vector_access, constants)?;
                    Ok(NodeReference {
                        node_type: NodeType::CONST,
                        index: constant_index,
                    })
                }
                NodeType::VAR => {
                    let public_input_index = get_public_input_index(ir, vector_access)?;
                    Ok(NodeReference {
                        node_type: NodeType::VAR,
                        index: public_input_index,
                    })
                }
                _ => Err(ConstraintEvaluationError::InvalidOperation(
                    "Invalid node type: only CONST and VAR allowed".to_string(),
                )),
            }
        }
        MatrixAccess(matrix_access) => {
            let constant_index = get_constant_index_by_matrix_access(ir, matrix_access, constants)?;
            Ok(NodeReference {
                node_type: NodeType::CONST,
                index: constant_index,
            })
        }
        Rand(rand_index) => {
            let index = get_random_value_index(ir, *rand_index);
            Ok(NodeReference {
                node_type: NodeType::VAR,
                index,
            })
        }
        Add(l, r) => parse_recursive_boundary_expression(
            ir,
            (l, r),
            "ADD".to_string(),
            constants,
            const_public_type_map,
            expressions,
        ),
        Sub(l, r) => parse_recursive_boundary_expression(
            ir,
            (l, r),
            "SUB".to_string(),
            constants,
            const_public_type_map,
            expressions,
        ),
        Mul(l, r) => parse_recursive_boundary_expression(
            ir,
            (l, r),
            "MUL".to_string(),
            constants,
            const_public_type_map,
            expressions,
        ),

        Exp(_i, _degree) => todo!(),
    }
}
