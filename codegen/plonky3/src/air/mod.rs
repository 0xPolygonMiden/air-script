mod boundary_constraints;
mod graph;
use graph::Codegen;
mod integrity_constraints;

use air_ir::Air;

use super::Scope;
use crate::air::{
    boundary_constraints::add_main_boundary_constraints,
    integrity_constraints::add_main_integrity_constraints,
};

// HELPERS TO GENERATE AN IMPLEMENTATION OF THE PLONKY3 AIR TRAIT
// ================================================================================================

/// Updates the provided scope with a new Air struct and Plonky3 Air trait implementation
/// which are equivalent the provided AirIR.
pub(super) fn add_air(scope: &mut Scope, ir: &Air) {
    let name = ir.name();

    // add the constants needed (outside any traits for object safety).
    add_constants(scope, ir);

    // add the Air struct and its base implementation.
    add_air_struct(scope, name);

    // add AirScriptAir trait implementation for the provided AirIR.
    add_air_script_trait(scope, ir, name);

    // add Plonky3 AirBuilder trait implementation for the provided AirIR.
    add_air_trait(scope, name);
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
    let num_alpha_challenges = ir.num_random_values;

    let constants = [
        format!("pub const MAIN_WIDTH: usize = {main_width};"),
        format!("pub const AUX_WIDTH: usize = {aux_width};"),
        format!("pub const NUM_PERIODIC_VALUES: usize = {num_periodic_values};"),
        format!("pub const PERIOD: usize = {period};"),
        format!("pub const NUM_PUBLIC_VALUES: usize = {num_public_values};"),
        format!("pub const NUM_ALPHA_CHALLENGES: usize = {num_alpha_challenges};"),
    ];

    scope.raw(constants.join("\n"));
}

/// Updates the provided scope with a custom Air struct.
fn add_air_struct(scope: &mut Scope, name: &str) {
    // define the custom Air struct.
    scope.new_struct(name).vis("pub");

    // add the custom BaseAir implementation block
    let base_air_impl = scope.new_impl(name).generic("F").impl_trait("BaseAir<F>");
    base_air_impl.new_fn("width").arg_ref_self().ret("usize").line("MAIN_WIDTH");

    // add the custom BaseAirWithPublicValues implementation block
    let base_air_with_public_values_impl =
        scope.new_impl(name).generic("F").impl_trait("BaseAirWithPublicValues<F>");
    base_air_with_public_values_impl
        .new_fn("num_public_values")
        .arg_ref_self()
        .ret("usize")
        .line("NUM_PUBLIC_VALUES");
}

fn add_air_script_trait(scope: &mut Scope, ir: &Air, name: &str) {
    // add the custom AirScriptAir implementation block
    let air_script_impl = scope
        .new_impl(name)
        .generic("F: Field")
        .generic("AB: AirScriptBuilder<F = F>")
        .impl_trait("AirScriptAir<F, AB>");

    // add the aux_width function
    air_script_impl
        .new_fn("aux_width")
        .arg_ref_self()
        .ret("usize")
        .line("AUX_WIDTH");

    // add the num_alpha_challenges function
    air_script_impl
        .new_fn("num_alpha_challenges")
        .arg_ref_self()
        .ret("usize")
        .line("NUM_ALPHA_CHALLENGES");

    // add the periodic_table function
    let periodic_table_func = air_script_impl
        .new_fn("periodic_table")
        .arg_ref_self()
        //.ret("&'static [&'static [F]]");
        .ret("Vec<Vec<F>>");
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

    // add the eval function
    let eval_func = air_script_impl.new_fn("eval").arg_ref_self().arg("builder", "&mut AB");
    eval_func.line("let main = builder.main();");
    eval_func.line("let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect(\"Wrong number of public values\");");
    eval_func.line("let periodic_values: [_; NUM_PERIODIC_VALUES] = builder.periodic_evals().try_into().expect(\"Wrong number of periodic values\");");
    eval_func.line("let (main_current, main_next) = (");
    eval_func.line("    main.row_slice(0).unwrap(),");
    eval_func.line("    main.row_slice(1).unwrap(),");
    eval_func.line(");");

    eval_func.line("");
    eval_func.line("// Main boundary constraints");
    add_main_boundary_constraints(eval_func, ir);

    eval_func.line("");
    eval_func.line("// Main integrity/transition constraints");
    add_main_integrity_constraints(eval_func, ir);
}

/// Updates the provided scope with the custom Air struct and an Air trait implementation based on
/// the provided AirIR.
fn add_air_trait(scope: &mut Scope, name: &str) {
    // add the implementation block for the Air trait.
    let air_impl = scope.new_impl(name).generic("AB: AirScriptBuilder").impl_trait("Air<AB>");

    let eval_func = air_impl.new_fn("eval").arg_ref_self().arg("builder", "&mut AB");
    eval_func.line("<Self as AirScriptAir<AB::F, AB>>::eval(self, builder);");
}
