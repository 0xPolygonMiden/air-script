use super::{AirIR, Codegen, Expression, Impl};

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
    get_assertions.line("let last_step = self.last_step();");

    // add the boundary constraints
    // TODO: need to do something clever here to get the trace column, since the constraint was combined in the graph
    // maybe it's worth representing this in two halves?
    for (col_idx, constraint) in ir.boundary_constraints(0) {
        let assertion = format!(
            "result.push(Assertion::single({}, {}, {}));",
            col_idx,
            constraint.domain(), // TODO: get the step number from the constraint domain
            constraint.to_string(ir, false)
        );
        get_assertions.line(assertion);
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
    get_assertions.line("let last_step = self.last_step();");

    // TODO: update this analagously to above
    // add the constraints for the auxiliary columns for the first boundary.
    for (col_idx, constraint) in ir.boundary_constraints(1) {
        let assertion = format!(
            "result.push(Assertion::single({}, 0, {}));",
            col_idx,
            constraint.to_string(ir, true)
        );
        get_aux_assertions.line(assertion);
    }

    // return the result
    get_aux_assertions.line("result");
}

// RUST STRING GENERATION
// ================================================================================================

/// Code generation trait for generating Rust code strings from boundary constraint expressions.
impl Codegen for Expression {
    // TODO: Only add parentheses in Add/Sub/Mul/Exp if the expression is an arithmetic operation.
    fn to_string(&self, ir: &AirIR, is_aux_constraint: bool) -> String {
        match self {
            Self::Const(value) => {
                if is_aux_constraint {
                    format!("E::from({value}_u64)")
                } else {
                    format!("Felt::new({value})")
                }
            }
            // TODO: Check element type and cast accordingly.
            Self::Elem(ident) => {
                if is_aux_constraint {
                    format!("E::from({ident})")
                } else {
                    format!("{ident}")
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
                format!("aux_rand_elements.get_segment_elements(0)[{index}]")
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
                if let Self::Const(rhs) = **rhs {
                    format!("({}).exp({})", lhs.to_string(ir, is_aux_constraint), rhs)
                } else {
                    todo!()
                }
            }
            _ => panic!("boundary constraint expressions cannot reference the trace"),
        }
    }
}
