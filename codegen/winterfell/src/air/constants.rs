use super::{AirIR, ConstantBinding, ConstantValueExpr, Scope};

/// Updates the provided scope with constant declarations.
pub(super) fn add_constants(scope: &mut Scope, ir: &AirIR) {
    let constants = ir.constants();
    let mut consts = vec![];
    for constant in constants {
        let const_str = match constant.value() {
            ConstantValueExpr::Scalar(_) => {
                format!(
                    "const {}: Felt = {};",
                    constant.name(),
                    constant.to_string()
                )
            }
            ConstantValueExpr::Vector(vector) => format!(
                "const {}: [Felt; {}] = {};",
                constant.name(),
                vector.len(),
                constant.to_string()
            ),
            ConstantValueExpr::Matrix(matrix) => format!(
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

impl Codegen for ConstantBinding {
    fn to_string(&self) -> String {
        match self.value() {
            ConstantValueExpr::Scalar(scalar_const) => match scalar_const {
                0 => "Felt::ZERO".to_string(),
                1 => "Felt::ONE".to_string(),
                _ => {
                    format!("Felt::new({scalar_const})")
                }
            },
            ConstantValueExpr::Vector(vector_const) => format!(
                "[{}]",
                vector_const
                    .iter()
                    .map(|val| match val {
                        0 => "Felt::ZERO".to_string(),
                        1 => "Felt::ONE".to_string(),
                        _ => {
                            format!("Felt::new({val})")
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ConstantValueExpr::Matrix(matrix_const) => {
                let mut rows = vec![];
                for row in matrix_const {
                    rows.push(format!(
                        "[{}]",
                        row.iter()
                            .map(|val| match val {
                                0 => "Felt::ZERO".to_string(),
                                1 => "Felt::ONE".to_string(),
                                _ => {
                                    format!("Felt::new({val})")
                                }
                            })
                            .collect::<Vec<String>>()
                            .join(", "),
                    ))
                }
                format!("[{}]", rows.join(", "))
            }
        }
    }
}
