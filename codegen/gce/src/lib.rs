use ir::{
    ast::constants::ConstantType::*,
    transition_stmts::{ConstantValue, Operation},
    AirIR, BoundaryExpr,
};

mod boundary_constraints;
use boundary_constraints::set_boundary_expressions_and_outputs;

mod error;
use error::ConstraintEvaluationError;

mod helpers;
use helpers::{acumulate_constants, count_boundary_rand_values, Expression, NodeType};

mod transition_constraints;
use transition_constraints::{set_transition_expressions, set_transition_outputs};

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;

/// Holds data for JSON generation
#[derive(Default, Debug)]
pub struct GCECodeGenerator {
    num_polys: u16,
    num_variables: usize,
    constants: Vec<u64>,
    expressions: Vec<Expression>,
    outputs: Vec<usize>,
}

impl GCECodeGenerator {
    pub fn new(ir: &AirIR, extension_degree: u8) -> Result<Self, ConstraintEvaluationError> {
        // vector of all boundary constraints vectors
        let boundary_constraints_vec = [
            &ir.main_first_boundary_constraints(),
            &ir.main_last_boundary_constraints(),
            &ir.aux_first_boundary_constraints(),
            &ir.aux_last_boundary_constraints(),
        ];

        // maps indexes in Node vector in AlgebraicGraph and in `expressions` JSON array
        let mut expressions_map = BTreeMap::new();

        // maps names of named constants and public inputs to their NodeType
        // Only CONST or VAR allowed
        let mut const_public_type_map = BTreeMap::new();

        // vector of expression nodes
        let mut expressions = Vec::new();

        // vector of `expressions` indexes
        let mut outputs = Vec::new();

        let num_polys = set_num_polys(ir, extension_degree);
        let num_variables =
            set_num_variables(ir, &mut const_public_type_map, boundary_constraints_vec);

        // TODO #1: get rid of the vector and push values directly into result string
        // constants from Constants AirIR field
        // TODO #2: currently I add all found constants in the vector. Should I add only unique ones,
        // since I'll get constants by their value, not index?
        let constants = set_constants(ir, &mut const_public_type_map, boundary_constraints_vec);

        set_transition_expressions(ir, &mut expressions, &constants, &mut expressions_map)?;
        set_transition_outputs(ir, &mut outputs, &expressions_map)?;
        set_boundary_expressions_and_outputs(
            ir,
            boundary_constraints_vec,
            &mut expressions,
            &mut outputs,
            &constants,
            &const_public_type_map,
        )?;

        Ok(GCECodeGenerator {
            num_polys,
            num_variables,
            constants,
            expressions,
            outputs,
        })
    }

    /// Generates constraint evaluation JSON file
    pub fn generate(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all("{\n".as_bytes())?;
        file.write_all(format!("\t\"num_polys\": {},\n", self.num_polys).as_bytes())?;
        file.write_all(format!("\t\"num_variables\": {},\n", self.num_variables).as_bytes())?;
        file.write_all(format!("\t\"constants\": {:?},\n", self.constants).as_bytes())?;
        file.write_all(format!("\t\"expressions\": [\n\t\t{}", self.expressions[0]).as_bytes())?;
        for expr in self.expressions.iter().skip(1) {
            file.write_all(format!(",\n\t\t{}", expr).as_bytes())?;
        }
        file.write_all("\n\t],\n".as_bytes())?;
        file.write_all(format!("\t\"outputs\": {:?}\n", self.outputs).as_bytes())?;

        file.write_all("}\n".as_bytes())?;
        Ok(())
    }
}

// HELPER FUNCTIONS
// ================================================================================================

fn set_num_polys(ir: &AirIR, extension_degree: u8) -> u16 {
    // TODO: Should all aux columns be extended to be quadratic or cubic?
    let num_polys_vec = ir.num_polys();
    num_polys_vec
        .iter()
        .skip(1)
        .fold(num_polys_vec[0], |acc, &x| {
            acc + x * extension_degree as u16
        })
}

fn set_num_variables<'a>(
    ir: &'a AirIR,
    const_public_type_map: &mut BTreeMap<&'a str, NodeType>,
    boundary_constraints_vec: [&Vec<(usize, &BoundaryExpr)>; 4],
) -> usize {
    let mut num_variables = 0;
    // public inputs
    for input in ir.public_inputs() {
        num_variables += input.1;
        const_public_type_map.insert(input.0.as_str(), NodeType::VAR);
    }

    // TODO: how many random values can we have? Would them fit in u8?
    let mut max_random_values_index = 0;
    // random values from boundary constrains
    for constraints in boundary_constraints_vec {
        for (_, expr) in constraints {
            count_boundary_rand_values(expr, &mut max_random_values_index);
        }
    }

    // random values from transition constrains
    for expr in ir.transition_graph().nodes() {
        if let Operation::RandomValue(i) = expr.op() {
            max_random_values_index = max_random_values_index.max(*i + 1)
        }
    }

    num_variables + max_random_values_index
}

fn set_constants<'a>(
    ir: &'a AirIR,
    const_public_type_map: &mut BTreeMap<&'a str, NodeType>,
    boundary_constraints_vec: [&Vec<(usize, &BoundaryExpr)>; 4],
) -> Vec<u64> {
    let mut constants = Vec::new();
    for constant in ir.constants() {
        match constant.value() {
            Scalar(value) => {
                constants.push(*value);
            }
            Vector(values) => {
                for elem in values {
                    constants.push(*elem);
                }
                // not sure thet this approach is better
                // let mut local_values = values.clone();
                // constants.append(&mut local_values);
            }
            Matrix(values) => {
                for elem in values.iter().flatten() {
                    constants.push(*elem);
                }
            }
        }
        const_public_type_map.insert(constant.name().name(), NodeType::CONST);
    }
    // constants from boundary_constraints
    for constraints in boundary_constraints_vec {
        for (_, expr) in constraints {
            acumulate_constants(expr, &mut constants);
        }
    }

    // constants and random values from transition_constraints
    for node in ir.transition_graph().nodes() {
        match node.op() {
            Operation::Constant(ConstantValue::Inline(v)) => constants.push(*v),
            Operation::Exp(_, degree) => {
                if *degree == 0 {
                    constants.push(1); // constant needed for optimization, since node^0 is Const(1)
                } else {
                    constants.push(*degree as u64)
                }
            }
            _ => {}
        }
    }

    constants
}
