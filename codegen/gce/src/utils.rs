use super::error::ConstraintEvaluationError;
pub use air_script_core::{
    Constant, ConstantType, Expression, Identifier, IndexedTraceAccess, MatrixAccess,
    NamedTraceAccess, TraceSegment, Variable, VariableType, VectorAccess,
};
use ir::AirIR;

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

pub fn get_random_value_index(ir: &AirIR, rand_index: &usize) -> usize {
    ir.public_inputs().iter().map(|v| v.1).sum::<usize>() + rand_index
}

pub fn get_public_input_index(ir: &AirIR, name: &String, public_index: &usize) -> usize {
    ir.public_inputs()
        .iter()
        .take_while(|v| v.0 != *name)
        .map(|v| v.1)
        .sum::<usize>()
        + public_index
}
