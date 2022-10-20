use super::{AirIR, Impl};

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
    for (col_idx, constraint) in ir.main_first_boundary_constraints().iter().enumerate() {
        let assertion = format!(
            "result.push(Assertion::single({}, 0, {}));",
            col_idx,
            constraint.to_string()
        );
        get_assertions.line(assertion);
    }

    // add the constraints for the last boundary.
    let last_constraints = ir.main_last_boundary_constraints();
    if !last_constraints.is_empty() {
        for (col_idx, constraint) in last_constraints.iter().enumerate() {
            let assertion = format!(
                "result.push(Assertion::single({}, 0, {}));",
                col_idx,
                constraint.to_string()
            );
            get_assertions.line(assertion);
        }
    }

    // return the result
    get_assertions.line("result");
}

// RUST STRING GENERATION
// ================================================================================================

/// Code generation trait for generating Rust code strings from boundary constraint expressions.
trait Codegen {
    fn to_string(&self) -> String;
}

impl Codegen for BoundaryExpr {
    fn to_string(&self) -> String {
        match self {
            Self::Constant(value) => format!("Felt::new({})", value),
            Self::Add(lhs, rhs) => {
                format!("{} + {}", lhs.to_string(), rhs.to_string())
            }
            Self::Subtract(lhs, rhs) => {
                format!("{} - {}", lhs.to_string(), rhs.to_string())
            }
        }
    }
}
