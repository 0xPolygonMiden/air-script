use super::{AirIR, Codegen, Impl};
use ir::BoundaryExpr;

// HELPERS TO GENERATE THE WINTERFELL BOUNDARY CONSTRAINT METHODS
// ================================================================================================

/// Adds an implementation of the "get_assertions" method to the referenced Air implementation
/// based on the data in the provided AirIR.
pub(super) fn add_fn_get_assertions(impl_ref: &mut Impl, ir: &AirIR) {
    // define the function
    let get_assertions = impl_ref
        .new_fn("get_assertions")
        .arg_ref_self()
        .ret("Vec<Assertion<Felt>>");

    // declare the result vector to be returned.
    get_assertions.line("let mut result = Vec::new();");

    // add the constraints for the first boundary
    for (col_idx, constraint) in ir.main_first_boundary_constraints() {
        let assertion = format!(
            "result.push(Assertion::single({}, 0, {}));",
            col_idx,
            constraint.to_string(false)
        );
        get_assertions.line(assertion);
    }

    // add the constraints for the last boundary.
    let last_constraints = ir.main_last_boundary_constraints();

    if !last_constraints.is_empty() {
        get_assertions.line("let last_step = self.last_step();");
        for (col_idx, constraint) in last_constraints {
            let assertion = format!(
                "result.push(Assertion::single({}, last_step, {}));",
                col_idx,
                constraint.to_string(false)
            );
            get_assertions.line(assertion);
        }
    }

    // return the result
    get_assertions.line("result");
}

/// Adds an implementation of the "get_aux_assertions" method to the referenced Air implementation
/// based on the data in the provided AirIR.
pub(super) fn add_fn_get_aux_assertions(impl_ref: &mut Impl, ir: &AirIR) {
    // define the function
    let get_aux_assertions = impl_ref
        .new_fn("get_aux_assertions")
        .generic("E: FieldElement<BaseField = Felt>")
        .arg_ref_self()
        .arg("aux_rand_elements", "&AuxTraceRandElements<E>")
        .ret("Vec<Assertion<E>>");

    // declare the result vector to be returned.
    get_aux_assertions.line("let mut result = Vec::new();");

    // add the constraints for the auxiliary columns for the first boundary.
    for (col_idx, constraint) in ir.aux_first_boundary_constraints() {
        let assertion = format!(
            "result.push(Assertion::single({}, 0, {}));",
            col_idx,
            constraint.to_string(true)
        );
        get_aux_assertions.line(assertion);
    }

    let last_aux_constraints = ir.aux_last_boundary_constraints();

    if !last_aux_constraints.is_empty() {
        get_aux_assertions.line("let last_step = self.last_step();");
        // add the constraints for the auxiliary columns for the last boundary.
        for (col_idx, constraint) in last_aux_constraints {
            let assertion = format!(
                "result.push(Assertion::single({}, last_step, {}));",
                col_idx,
                constraint.to_string(true)
            );
            get_aux_assertions.line(assertion);
        }
    }

    // return the result
    get_aux_assertions.line("result");
}

// RUST STRING GENERATION
// ================================================================================================

/// Code generation trait for generating Rust code strings from boundary constraint expressions.
impl Codegen for BoundaryExpr {
    fn to_string(&self, is_aux_constraint: bool) -> String {
        match self {
            Self::Const(value) => {
                if is_aux_constraint {
                    format!("E::from({}_u64)", value)
                } else {
                    format!("Felt::new({})", value)
                }
            }
            Self::PubInput(name, index) => format!("self.{}[{}]", name, index),
            Self::Rand(index) => {
                format!("aux_rand_elements.get_segment_elements(0)[{}]", index)
            }
            Self::Add(lhs, rhs) => {
                format!(
                    "{} + {}",
                    lhs.to_string(is_aux_constraint),
                    rhs.to_string(is_aux_constraint)
                )
            }
            Self::Sub(lhs, rhs) => {
                let rhs = if is_arithmetic_expr(rhs) {
                    format!("({})", rhs.to_string(is_aux_constraint))
                } else {
                    rhs.to_string(is_aux_constraint)
                };
                format!("{} - {}", lhs.to_string(is_aux_constraint), rhs)
            }
            Self::Mul(lhs, rhs) => {
                let lhs = if is_arithmetic_expr(lhs) {
                    format!("({})", lhs.to_string(is_aux_constraint))
                } else {
                    lhs.to_string(is_aux_constraint)
                };
                let rhs = if is_arithmetic_expr(rhs) {
                    format!("({})", rhs.to_string(is_aux_constraint))
                } else {
                    rhs.to_string(is_aux_constraint)
                };
                format!("{} * {}", lhs, rhs)
            }
            Self::Exp(lhs, rhs) => {
                let lhs = if is_arithmetic_expr(lhs) {
                    format!("({})", lhs.to_string(is_aux_constraint))
                } else {
                    lhs.to_string(is_aux_constraint)
                };
                format!("{}.exp({})", lhs, rhs)
            }
        }
    }
}

/// Checks whether the boundary expression is an arithmetic operation.
fn is_arithmetic_expr(boundary_expr: &BoundaryExpr) -> bool {
    matches!(
        boundary_expr,
        BoundaryExpr::Add(_, _) | BoundaryExpr::Sub(_, _) | BoundaryExpr::Mul(_, _)
    )
}
