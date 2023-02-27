use super::{AirIR, Impl, PeriodicColumns};

pub(super) fn add_fn_get_periodic_column_values(impl_ref: &mut Impl, ir: &AirIR) {
    // define the function.
    let get_periodic_column_values = impl_ref
        .new_fn("get_periodic_column_values")
        .arg_ref_self()
        .ret("Vec<Vec<Felt>>");

    // output the periodic columns.
    get_periodic_column_values.line(ir.periodic_columns().to_string());
}

/// Code generation trait for generating Rust code strings from Periodic Columns.
trait Codegen {
    fn to_string(&self) -> String;
}

impl Codegen for PeriodicColumns {
    fn to_string(&self) -> String {
        let mut columns = vec![];
        for column in self {
            let mut rows = vec![];
            for row in column {
                match row {
                    0 => {
                        rows.push("Felt::ZERO".to_string());
                    }
                    1 => {
                        rows.push("Felt::ONE".to_string());
                    }
                    _ => {
                        rows.push(format!("Felt::new({row})"));
                    }
                }
            }
            columns.push(format!("vec![{}]", rows.join(", ")));
        }
        format!("vec![{}]", columns.join(", "))
    }
}
