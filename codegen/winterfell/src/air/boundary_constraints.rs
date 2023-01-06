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
            constraint.to_string(ir, false)
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
                constraint.to_string(ir, false)
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
            constraint.to_string(ir, true)
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
                constraint.to_string(ir, true)
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
    // TODO: Only add parentheses in Add/Sub/Mul/Exp if the expression is an arithmetic operation.
    fn to_string(&self, ir: &AirIR, is_aux_constraint: bool) -> String {
        match self {
            Self::Const(value) => {
                if is_aux_constraint {
                    format!("E::from({}_u64)", value)
                } else {
                    format!("Felt::new({})", value)
                }
            }
            // TODO: Check element type and cast accordingly.
            Self::Elem(ident) => {
                if is_aux_constraint {
                    format!("E::from({})", ident)
                } else {
                    format!("{}", ident)
                }
            }
            Self::VectorAccess(vector_access) => {
                // check if vector_access is a public input
                // TODO: figure out a better way to handle this lookup.
                if ir
                    .public_inputs()
                    .iter()
                    .any(|input| input.0 == vector_access.name())
                {
                    format!("self.{}[{}]", vector_access.name(), vector_access.idx())
                } else if is_aux_constraint {
                    format!("E::from({}[{}])", vector_access.name(), vector_access.idx())
                } else {
                    format!("{}[{}]", vector_access.name(), vector_access.idx())
                }
            }
            Self::MatrixAccess(matrix_access) => {
                if is_aux_constraint {
                    format!(
                        "E::from({}[{}][{}])",
                        matrix_access.name(),
                        matrix_access.row_idx(),
                        matrix_access.col_idx()
                    )
                } else {
                    format!(
                        "{}[{}][{}]",
                        matrix_access.name(),
                        matrix_access.row_idx(),
                        matrix_access.col_idx()
                    )
                }
            }
            Self::Rand(index) => {
                format!("aux_rand_elements.get_segment_elements(0)[{}]", index)
            }
            Self::Add(lhs, rhs) => {
                format!(
                    "({}) + ({})",
                    lhs.to_string(ir, is_aux_constraint),
                    rhs.to_string(ir, is_aux_constraint)
                )
            }
            Self::Sub(lhs, rhs) => {
                format!(
                    "({}) - ({})",
                    lhs.to_string(ir, is_aux_constraint),
                    rhs.to_string(ir, is_aux_constraint)
                )
            }
            Self::Mul(lhs, rhs) => {
                format!(
                    "({}) * ({})",
                    lhs.to_string(ir, is_aux_constraint),
                    rhs.to_string(ir, is_aux_constraint)
                )
            }
            Self::Exp(lhs, rhs) => {
                format!("({}).exp({})", lhs.to_string(ir, is_aux_constraint), rhs)
            }
        }
    }
}
