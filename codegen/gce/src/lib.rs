use ir::{
    constraints::{ConstantValue, Operation},
    AirIR,
};

pub use air_script_core::{
    Constant, ConstantType, Expression, Identifier, IndexedTraceAccess, MatrixAccess,
    NamedTraceAccess, TraceSegment, Variable, VariableType, VectorAccess,
};
use std::fmt::Display;

mod error;
use error::ConstraintEvaluationError;

mod utils;

mod expressions;
use expressions::ExpressionsHandler;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;

/// Holds data for JSON generation
#[derive(Default, Debug)]
pub struct CodeGenerator {
    num_polys: u16,
    num_variables: usize,
    constants: Vec<u64>,
    expressions: Vec<ExpressionJson>,
    outputs: Vec<usize>,
}

impl CodeGenerator {
    pub fn new(ir: &AirIR, extension_degree: u8) -> Result<Self, ConstraintEvaluationError> {
        // maps indexes in Node vector in AlgebraicGraph and in `expressions` JSON array
        let mut expressions_map = BTreeMap::new();

        let num_polys = set_num_polys(ir, extension_degree);
        let num_variables = set_num_variables(ir);
        let constants = set_constants(ir);

        let mut expressions_handler = ExpressionsHandler::new(ir, &constants, &mut expressions_map);

        let mut expressions = expressions_handler.get_expressions()?;
        // vector of `expressions` indexes
        let outputs = expressions_handler.get_outputs(&mut expressions)?;

        Ok(CodeGenerator {
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
            file.write_all(format!(",\n\t\t{expr}").as_bytes())?;
        }
        file.write_all("\n\t],\n".as_bytes())?;
        file.write_all(format!("\t\"outputs\": {:?}\n", self.outputs).as_bytes())?;

        file.write_all("}\n".as_bytes())?;
        Ok(())
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Returns total number of trace columns according to provided extension degree.
/// The result is calculated as `number of main columns + (number of aux columns) * extension
/// degree`.
fn set_num_polys(ir: &AirIR, extension_degree: u8) -> u16 {
    // TODO: Should all aux columns be extended to be quadratic or cubic?
    let num_polys_vec = ir.segment_widths();
    num_polys_vec
        .iter()
        .skip(1)
        .fold(num_polys_vec[0], |acc, &x| {
            acc + x * extension_degree as u16
        })
}

/// Returns total number of public inputs and random values.
fn set_num_variables(ir: &AirIR) -> usize {
    let mut num_variables = 0;

    // public inputs
    for input in ir.public_inputs() {
        num_variables += input.1;
    }

    num_variables + ir.num_random_values() as usize
}

/// Returns a vector of all unique constants: named ones defined in `constants` section and inline
/// ones used in constraints calculation. Every value in vector or matrix considered as new
/// constant.
///
/// # Examples
///
/// Fragment of AIR script:
///
/// ```airscript
/// const A = 1
/// const B = [0, 1]
/// const C = [[1, 2], [2, 0]]
///
/// boundary_constraints:
///     enf a.first = 1
///     enf a.last = 5
/// ```
///
/// Result vector: `[1, 0, 2, 5]`
fn set_constants(ir: &AirIR) -> Vec<u64> {
    //named constants
    let mut constants = Vec::new();
    for constant in ir.constants() {
        match constant.value() {
            ConstantType::Scalar(value) => {
                if !constants.contains(value) {
                    constants.push(*value);
                }
            }
            ConstantType::Vector(values) => {
                for elem in values {
                    if !constants.contains(elem) {
                        constants.push(*elem);
                    }
                }
            }
            ConstantType::Matrix(values) => {
                for elem in values.iter().flatten() {
                    if !constants.contains(elem) {
                        constants.push(*elem);
                    }
                }
            }
        }
    }

    // inline constants
    for node in ir.constraint_graph().nodes() {
        match node.op() {
            Operation::Constant(ConstantValue::Inline(value)) => {
                if !constants.contains(value) {
                    constants.push(*value);
                }
            }
            Operation::Exp(_, degree) => {
                if *degree == 0 {
                    if !constants.contains(&1) {
                        constants.push(1); // constant needed for optimization, since node^0 is Const(1)
                    }
                } else if !constants.contains(&(*degree as u64)) {
                    constants.push(*degree as u64)
                }
            }
            _ => {}
        }
    }

    constants
}

/// Stroes node type required in [NodeReference] struct
#[derive(Debug, Clone)]
pub enum NodeType {
    Pol,
    PolNext,
    Var,
    Const,
    Expr,
}

impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pol => write!(f, "POL"),
            Self::PolNext => write!(f, "POL_NEXT"),
            Self::Var => write!(f, "VAR"),
            Self::Const => write!(f, "CONST"),
            Self::Expr => write!(f, "EXPR"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ExpressionOperation {
    Add,
    Sub,
    Mul,
}

impl Display for ExpressionOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "ADD"),
            Self::Sub => write!(f, "SUB"),
            Self::Mul => write!(f, "MUL"),
        }
    }
}

/// Stores data used in JSON generation
#[derive(Debug, Clone)]
pub struct NodeReference {
    pub node_type: NodeType,
    pub index: usize,
}

impl Display for NodeReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"type\": \"{}\", \"index\": {}}}",
            self.node_type, self.index
        )
    }
}

/// Stores data used in JSON generation
#[derive(Clone, Debug)]
pub struct ExpressionJson {
    pub op: ExpressionOperation,
    pub lhs: NodeReference,
    pub rhs: NodeReference,
}

impl Display for ExpressionJson {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"op\": \"{}\", \"lhs\": {}, \"rhs\": {}}}",
            self.op, self.lhs, self.rhs
        )
    }
}
