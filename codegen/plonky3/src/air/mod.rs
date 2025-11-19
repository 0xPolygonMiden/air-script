mod boundary_constraints;
mod graph;
mod integrity_constraints;

use air_ir::Air;

use super::Scope;
use crate::air::{
    boundary_constraints::{add_aux_boundary_constraints, add_main_boundary_constraints},
    integrity_constraints::{add_aux_integrity_constraints, add_main_integrity_constraints},
};

#[derive(Debug, Clone, Copy)]
pub enum ElemType {
    Base,
    Ext,
}

// HELPERS TO GENERATE AN IMPLEMENTATION OF THE PLONKY3 AIR TRAIT
// ================================================================================================

/// Updates the provided scope with a new Air struct and Plonky3 Air trait implementation
/// which are equivalent the provided AirIR.
pub(super) fn add_air(scope: &mut Scope, ir: &Air) {
    let name = ir.name();

    // add the constants needed (outside any traits for object safety).
    add_constants(scope, ir);

    // add the Air struct and its base implementation.
    add_air_struct(scope, ir, name);

    // add AirScriptAir trait implementation for the provided AirIR.
    add_air_script_trait(scope, ir, name);

    // add Plonky3 AirBuilder trait implementation for the provided AirIR.
    /* add_air_trait(scope, name); */
}

/// Updates the provided scope with constants needed for the custom Air struct and trait
/// implementations.
fn add_constants(scope: &mut Scope, ir: &Air) {
    let main_width = ir.trace_segment_widths[0];
    let aux_width = ir.trace_segment_widths.get(1).cloned().unwrap_or(0);
    let num_periodic_values = ir.periodic_columns().count();
    let period = ir.periodic_columns().map(|col| col.period()).max().unwrap_or(0);
    let num_public_values =
        ir.public_inputs().map(|public_input| public_input.size()).sum::<usize>();
    let num_beta_challenges = ir.num_random_values.saturating_sub(1);

    let constants = [
        format!("pub const MAIN_WIDTH: usize = {main_width};"),
        format!("pub const AUX_WIDTH: usize = {aux_width};"),
        format!("pub const NUM_PERIODIC_VALUES: usize = {num_periodic_values};"),
        format!("pub const PERIOD: usize = {period};"),
        format!("pub const NUM_PUBLIC_VALUES: usize = {num_public_values};"),
        format!("pub const NUM_BETA_CHALLENGES: usize = {num_beta_challenges};"),
    ];

    scope.raw(constants.join("\n"));
}

/// Updates the provided scope with a custom Air struct.
fn add_air_struct(scope: &mut Scope, ir: &Air, name: &str) {
    // define the custom Air struct.
    scope.new_struct(name).vis("pub");

    // add the custom MidenAir implementation block
    let miden_air_impl = scope
        .new_impl(name)
        .generic("F")
        .generic("EF")
        .bound("F", "Field")
        .bound("EF", "ExtensionField<F>")
        .impl_trait("MidenAir<F, EF>");

    miden_air_impl.new_fn("width").arg_ref_self().ret("usize").line("MAIN_WIDTH");

    /*// add the custom BaseAirWithPublicValues implementation block
    let base_air_with_public_values_impl =
        scope.new_impl(name).generic("F").impl_trait("BaseAirWithPublicValues<F>");
    base_air_with_public_values_impl
        .new_fn("num_public_values")
        .arg_ref_self()
        .ret("usize")
        .line("NUM_PUBLIC_VALUES");
    */

    // add the eval function
    let eval_func = miden_air_impl
        .new_fn("eval")
        .generic("AB")
        .bound("AB", "MidenAirBuilder<F = F, EF = EF>")
        .arg_ref_self()
        .arg("builder", "&mut AB");
    eval_func.line("let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect(\"Wrong number of public values\");");
    //eval_func.line("let periodic_values: [_; NUM_PERIODIC_VALUES] =
    // builder.periodic_evals().try_into().expect(\"Wrong number of periodic values\");");
    eval_func.line("let preprocessed = builder.preprocessed();");
    eval_func.line("let periodic_values = preprocessed.row_slice(0).unwrap();");

    eval_func.line("let main = builder.main();");
    eval_func.line("let (main_current, main_next) = (");
    eval_func.line("    main.row_slice(0).unwrap(),");
    eval_func.line("    main.row_slice(1).unwrap(),");
    eval_func.line(");");

    // Only had aux if there are random values
    if ir.num_random_values > 0 {
        eval_func.line("let (alpha, beta_challenges) = builder.permutation_randomness().split_first().unwrap();");
        eval_func.line("let beta_challenges: [_; NUM_BETA_CHALLENGES] = beta_challenges.try_into().expect(\"Wrong number of beta challenges\");");
        eval_func.line("let aux_bus_boundary_values: [_; AUX_WIDTH] = builder.aux_bus_boundary_values().try_into().expect(\"Wrong number of aux bus boundary values\");");
        eval_func.line("let aux = builder.permutation();");
        eval_func.line("let (aux_current, aux_next) = (");
        eval_func.line("    aux.row_slice(0).unwrap(),");
        eval_func.line("    aux.row_slice(1).unwrap(),");
        eval_func.line(");");
    }

    add_main_boundary_constraints(eval_func, ir);

    add_main_integrity_constraints(eval_func, ir);

    add_aux_boundary_constraints(eval_func, ir);

    add_aux_integrity_constraints(eval_func, ir);
}

fn add_air_script_trait(scope: &mut Scope, ir: &Air, name: &str) {
    // add the custom AirScriptAir implementation block
    let air_script_impl = scope
        .new_impl(name)
        .generic("F: Field")
        .generic("EF: ExtensionField<F>")
        .impl_trait("AirScriptAir<F, EF>");

    // add the num_beta_challenges function
    air_script_impl
        .new_fn("num_beta_challenges")
        .arg_ref_self()
        .ret("usize")
        .line("NUM_BETA_CHALLENGES");

    // add the periodic_table function
    let periodic_table_func =
        air_script_impl.new_fn("periodic_table").arg_ref_self().ret("Vec<Vec<F>>");
    if ir.periodic_columns().count() == 0 {
        periodic_table_func.line("vec![]");
    } else {
        periodic_table_func.line("vec![");
        for col in ir.periodic_columns() {
            let values_str = col.values
                .iter()
                .map(|v| format!("F::from_u64({v})")) // or use a custom formatter if needed
                .collect::<Vec<_>>()
                .join(", ");
            periodic_table_func.line(format!("    vec![{values_str}],"));
        }
        periodic_table_func.line("]");
    }
}
