use super::error::ConstraintEvaluationError;
use ir::{
    ast::{constants::ConstantType, MatrixAccess, VectorAccess},
    AirIR, BoundaryExpr,
};
use std::fmt::Display;

// I think we can allow non camel case type since we translate it directly to string in node
// reference type, where we don't use camel case
/// Stroes node type required in [NodeReference] struct
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub enum NodeType {
    POL,
    POL_NEXT,
    VAR,
    CONST,
    EXPR,
}

/// Stores data used in JSON generation
#[derive(Debug, Clone)]
pub struct NodeReference {
    pub node_type: NodeType,
    pub index: usize,
}

// TODO: change String to &str (Or should I create another enum?)
/// Stores data used in JSON generation
#[derive(Debug)]
pub struct Expression {
    pub op: String,
    pub lhs: NodeReference,
    pub rhs: NodeReference,
}

impl Display for NodeReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"type\": \"{:?}\", \"index\": {}}}",
            self.node_type, self.index
        )
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"op\": \"{}\", \"lhs\": {}, \"rhs\": {}}}",
            self.op, self.lhs, self.rhs
        )
    }
}

/// Counts overall number of random values in boundary constraints
pub fn count_boundary_rand_values<'a>(
    expr: &'a BoundaryExpr,
    max_random_values_index: &'a mut usize,
) {
    use BoundaryExpr::*;
    match expr {
        Rand(i) => *max_random_values_index = *max_random_values_index.max(&mut (*i + 1)),
        Add(l, r) | Sub(l, r) | Mul(l, r) => {
            count_boundary_rand_values(l, max_random_values_index);
            count_boundary_rand_values(r, max_random_values_index);
        }
        _ => {}
    }
}

/// Pushes inline constants found in boundary expression to the `constants` vector
pub fn accumulate_constants(expr: &BoundaryExpr, constants: &mut Vec<u64>) {
    use BoundaryExpr::*;
    match expr {
        Const(v) => constants.push(*v),
        Add(l, r) | Sub(l, r) | Mul(l, r) => {
            accumulate_constants(l, constants);
            accumulate_constants(r, constants);
        }
        Exp(i, degree) => {
            if *degree == 0 {
                constants.push(1); // constant needed for optimization, since node^0 is Const(1)
            } else {
                constants.push(*degree);
            }
            accumulate_constants(i, constants);
        }
        _ => {}
    }
}

/// Returns index of the constant found in the `constants` array by its value
pub fn get_constant_index_by_value(
    v: u64,
    constants: &[u64],
) -> Result<usize, ConstraintEvaluationError> {
    constants
        .iter()
        .position(|&x| x == v)
        .ok_or_else(|| ConstraintEvaluationError::constant_not_found(&v.to_string()))
}

/// Returns index of the constant found in the `constants` array by its `name`
pub fn get_constant_index_by_name(
    ir: &AirIR,
    name: &String,
    constants: &[u64],
) -> Result<usize, ConstraintEvaluationError> {
    let constant = ir
        .constants()
        .iter()
        .find(|v| v.name().name() == name)
        .ok_or_else(|| ConstraintEvaluationError::constant_not_found(name))?;
    let value = match constant.value() {
        ConstantType::Scalar(s) => Ok(*s),
        _ => Err(ConstraintEvaluationError::invalid_constant_type(
            name, "Scalar",
        )),
    }?;
    get_constant_index_by_value(value, constants)
}

/// Returns index of the constant found in the `constants` array by its vector access (name and
/// index)
pub fn get_constant_index_by_vector_access(
    ir: &AirIR,
    vector_access: &VectorAccess,
    constants: &[u64],
) -> Result<usize, ConstraintEvaluationError> {
    let constant = ir
        .constants()
        .iter()
        .find(|v| v.name().name() == vector_access.name())
        .ok_or_else(|| ConstraintEvaluationError::constant_not_found(vector_access.name()))?;
    let value = match constant.value() {
        ConstantType::Vector(v) => Ok(v[vector_access.idx()]),
        _ => Err(ConstraintEvaluationError::invalid_constant_type(
            vector_access.name(),
            "Vector",
        )),
    }?;
    get_constant_index_by_value(value, constants)
}

/// Returns index of the constant found in the `constants` array by its matrix access (name and
/// indexes)
pub fn get_constant_index_by_matrix_access(
    ir: &AirIR,
    matrix_access: &MatrixAccess,
    constants: &[u64],
) -> Result<usize, ConstraintEvaluationError> {
    let constant = ir
        .constants()
        .iter()
        .find(|v| v.name().name() == matrix_access.name())
        .ok_or_else(|| ConstraintEvaluationError::constant_not_found(matrix_access.name()))?;

    let value = match constant.value() {
        ConstantType::Matrix(m) => Ok(m[matrix_access.row_idx()][matrix_access.col_idx()]),
        _ => Err(ConstraintEvaluationError::invalid_constant_type(
            matrix_access.name(),
            "Matrix",
        )),
    }?;
    get_constant_index_by_value(value, constants)
}

/// Returns index of the public input value found in the merged public inputs array by its vector
/// access (name and index)
pub fn get_public_input_index(
    ir: &AirIR,
    vector_access: &VectorAccess,
) -> Result<usize, ConstraintEvaluationError> {
    let mut accumulative_index = 0;
    for public_input in ir.public_inputs() {
        if vector_access.name() == public_input.0 {
            accumulative_index += vector_access.idx();
            break;
        }
        accumulative_index += public_input.1;
    }

    if accumulative_index == ir.public_inputs().iter().map(|v| v.1).sum() {
        return Err(ConstraintEvaluationError::public_input_not_found(
            vector_access.name(),
        ));
    }

    Ok(accumulative_index)
}

pub fn get_random_value_index(ir: &AirIR, rand_index: usize) -> usize {
    ir.public_inputs().iter().map(|v| v.1).sum::<usize>() + rand_index
}
