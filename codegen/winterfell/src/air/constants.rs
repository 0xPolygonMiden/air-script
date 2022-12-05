use super::Scope;
use ir::{ast::constants::ConstantType, AirIR, Constant};

/// Updates the provided scope with constant declarations.
pub(super) fn add_constants(scope: &mut Scope, ir: &AirIR) {
    let constants = ir.constants();
    for constant in constants {
        scope.raw(format!(
            "const {} = {};",
            constant.name(),
            constant.to_string()
        ));
    }
}

/// Code generation trait for generating Rust code strings from Constants.
trait Codegen {
    fn to_string(&self) -> String;
}

impl Codegen for Constant {
    fn to_string(&self) -> String {
        match self.value() {
            ConstantType::Scalar(scalar_const) => scalar_const.to_string(),
            ConstantType::Vector(vector_const) => vector_const
                .iter()
                .map(|val| format!("{}", val))
                .collect::<Vec<String>>()
                .join(", "),
            ConstantType::Matrix(matrix_const) => {
                let mut rows = vec![];
                for row in matrix_const {
                    rows.push(
                        row.iter()
                            .map(|val| format!("{}", val))
                            .collect::<Vec<String>>()
                            .join(", "),
                    )
                }
                format!("vec![{}]", rows.join(", "))
            }
        }
    }
}
