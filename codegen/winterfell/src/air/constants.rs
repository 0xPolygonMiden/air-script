use super::Scope;
use ir::{ast::constants::ConstantType, AirIR, Constant};

/// Updates the provided scope with constant declarations.
pub(super) fn add_constants(scope: &mut Scope, ir: &AirIR) {
    let constants = ir.constants();
    let mut consts = vec![];
    for constant in constants {
        let const_str = match constant.value() {
            ConstantType::Scalar(_) => {
                format!(
                    "const {}: Felt = {};",
                    constant.name(),
                    constant.to_string()
                )
            }
            ConstantType::Vector(vector) => format!(
                "const {}: [Felt; {}] = {};",
                constant.name(),
                vector.len(),
                constant.to_string()
            ),
            ConstantType::Matrix(matrix) => format!(
                "const {}: [[Felt; {}]; {}] = {};",
                constant.name(),
                matrix[0].len(),
                matrix.len(),
                constant.to_string()
            ),
        };
        consts.push(const_str);
    }
    scope.raw(consts.join("\n"));
}

/// Code generation trait for generating Rust code strings from Constants.
trait Codegen {
    fn to_string(&self) -> String;
}

impl Codegen for Constant {
    fn to_string(&self) -> String {
        match self.value() {
            ConstantType::Scalar(scalar_const) => {
                format!("Felt::new({})", scalar_const)
            }
            ConstantType::Vector(vector_const) => format!(
                "[{}]",
                vector_const
                    .iter()
                    .map(|val| format!("Felt::new({})", val))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ConstantType::Matrix(matrix_const) => {
                let mut rows = vec![];
                for row in matrix_const {
                    rows.push(format!(
                        "[{}]",
                        row.iter()
                            .map(|val| format!("Felt::new({})", val))
                            .collect::<Vec<String>>()
                            .join(", "),
                    ))
                }
                format!("[{}]", rows.join(", "))
            }
        }
    }
}
