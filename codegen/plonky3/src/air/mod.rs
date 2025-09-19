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

    // add the Air struct and its base implementation.
    add_air_struct(scope, ir, name);

    // add Plonky3 AirBuilder trait implementation for the provided AirIR.
    add_air_trait(scope, ir, name);
}

/// Updates the provided scope with a custom Air struct.
fn add_air_struct(scope: &mut Scope, ir: &Air, name: &str) {
    scope.raw(format!("pub const NUM_COLUMNS: usize = {};", ir.trace_segment_widths[0]));

    let num_public_values =
        ir.public_inputs().map(|public_input| public_input.size()).sum::<usize>();
    scope.raw(format!("pub const NUM_PUBLIC_VALUES: usize = {num_public_values};"));

    // define the custom Air struct.
    scope.new_struct(name).vis("pub");

    // add the custom BaseAir implementation block
    let base_air_impl = scope.new_impl(name).generic("F").impl_trait("BaseAir<F>");
    base_air_impl.new_fn("width").arg_ref_self().ret("usize").line("NUM_COLUMNS");

    // add the custom BaseAirWithPublicValues implementation block
    let base_air_with_public_values_impl =
        scope.new_impl(name).generic("F").impl_trait("BaseAirWithPublicValues<F>");
    base_air_with_public_values_impl
        .new_fn("num_public_values")
        .arg_ref_self()
        .ret("usize")
        .line("NUM_PUBLIC_VALUES");
}

/// Updates the provided scope with the custom Air struct and an Air trait implementation based on
/// the provided AirIR.
fn add_air_trait(scope: &mut Scope, ir: &Air, name: &str) {
    // add the implementation block for the Air trait.
    let air_impl = scope
        .new_impl(name)
        .generic("AB: AirBuilderWithPublicValues")
        .impl_trait("Air<AB>");

    let eval_func = air_impl.new_fn("eval").arg_ref_self().arg("builder", "&mut AB");
    eval_func.line("let main = builder.main();");
    eval_func.line("let public_values = builder.public_values().to_vec();");
    eval_func.line("let (main_current, main_next) = (");
    eval_func.line("    main.row_slice(0).unwrap(),");
    eval_func.line("    main.row_slice(1).unwrap(),");
    eval_func.line(");");

    add_main_boundary_constraints(eval_func, ir);

    add_main_integrity_constraints(eval_func, ir);
}
