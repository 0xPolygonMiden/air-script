use super::Scope;
use ir::{ast::constants::ConstantType, AirIR, Constant};

/// Updates the provided scope with constant declarations.
pub(super) fn add_constants(scope: &mut Scope, ir: &AirIR) {
    let constants = ir.constants();
    let mut consts = vec![];
    for constant in constants {
        let const_str = match constant.value() {
            ConstantType::Scalar(_) => {
                format!("const {}: u64 = {};", constant.name(), constant.to_string())
            }
            ConstantType::Vector(_) => format!(
                "const {}: Vec<u64> = {};",
                constant.name(),
                constant.to_string()
            ),
            ConstantType::Matrix(_) => format!(
                "const {}: Vec<Vec<u64>> = {};",
                constant.name(),
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
            ConstantType::Scalar(scalar_const) => scalar_const.to_string(),
            ConstantType::Vector(vector_const) => format!(
                "vec![{}]",
                vector_const
                    .iter()
                    .map(|val| val.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ConstantType::Matrix(matrix_const) => {
                let mut rows = vec![];
                for row in matrix_const {
                    rows.push(format!(
                        "vec![{}]",
                        row.iter()
                            .map(|val| val.to_string())
                            .collect::<Vec<String>>()
                            .join(", "),
                    ))
                }
                format!("vec![{}]", rows.join(", "))
            }
        }
    }
}
