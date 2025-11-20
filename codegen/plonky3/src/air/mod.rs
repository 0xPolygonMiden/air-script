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

    // add the width function
    miden_air_impl.new_fn("width").arg_ref_self().ret("usize").line("MAIN_WIDTH");

    // add the preprocessed_trace_function if needed (for now, never needed)
    // miden_air_impl.new_fn("preprocessed_trace").arg_ref_self().ret("Option<RowMajorMatrix<F>>").
    // line("None");

    // add the num_public_values function if needed
    if ir.periodic_columns().count() > 0 {
        // add the custom BaseAirWithPublicValues implementation block
        miden_air_impl
            .new_fn("num_public_values")
            .arg_ref_self()
            .ret("usize")
            .line("NUM_PUBLIC_VALUES");
    }

    // add the periodic_table function if needed
    if ir.periodic_columns().count() > 0 {
        let periodic_table_func =
            miden_air_impl.new_fn("periodic_table").arg_ref_self().ret("Vec<Vec<F>>");
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

    // add the num_randomness and aux_width functions if needed
    if ir.num_random_values > 0 {
        miden_air_impl
            .new_fn("num_randomness")
            .arg_ref_self()
            .ret("usize")
            .line("1 + NUM_BETA_CHALLENGES");

        miden_air_impl.new_fn("aux_width").arg_ref_self().ret("usize").line("AUX_WIDTH");
    }

    // For now, don't provide the build_aux_trace and with_aux_builder functions.
    // TODO: add them

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
        eval_func.line("let (&alpha, beta_challenges) = builder.permutation_randomness().split_first().unwrap();");
        eval_func.line("let beta_challenges: [_; NUM_BETA_CHALLENGES] = beta_challenges.try_into().expect(\"Wrong number of randomness\");");
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
