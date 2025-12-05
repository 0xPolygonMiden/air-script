mod boundary_constraints;
mod graph;
mod integrity_constraints;

use air_ir::Air;

use super::Scope;
use crate::air::{
    boundary_constraints::{add_aux_boundary_constraints, add_main_boundary_constraints},
    graph::Codegen,
    integrity_constraints::{add_aux_integrity_constraints, add_main_integrity_constraints},
};

#[derive(Debug, Clone, Copy)]
pub enum ElemType {
    Base,
    Ext,
    ExtFieldElem,
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

    // add the aux trace generation utils if needed
    if ir.num_random_values > 0 {
        add_aux_trace_utils(scope, ir, name);
    }
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
    let max_beta_challenge_power = ir.num_random_values.saturating_sub(1);

    let constants = [
        format!("pub const MAIN_WIDTH: usize = {main_width};"),
        format!("pub const AUX_WIDTH: usize = {aux_width};"),
        format!("pub const NUM_PERIODIC_VALUES: usize = {num_periodic_values};"),
        format!("pub const PERIOD: usize = {period};"),
        format!("pub const NUM_PUBLIC_VALUES: usize = {num_public_values};"),
        format!("pub const MAX_BETA_CHALLENGE_POWER: usize = {max_beta_challenge_power};"),
    ];

    scope.raw(constants.join("\n"));
}

/// Updates the provided scope with a custom Air struct.
fn add_air_struct(scope: &mut Scope, ir: &Air, name: &str) {
    // define the custom Air struct.
    scope.new_struct(name).vis("pub");

    // add the custom MidenAir implementation block
    let miden_air_impl =
        scope.new_impl(name).generic("F").generic("EF").impl_trait("MidenAir<F, EF>");

    if ir.num_random_values > 0 || ir.periodic_columns().count() > 0 {
        miden_air_impl.bound("F", "Field").bound("EF", "ExtensionField<F>");
    }

    // add the width function
    miden_air_impl.new_fn("width").arg_ref_self().ret("usize").line("MAIN_WIDTH");

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
            .line("1 + MAX_BETA_CHALLENGE_POWER");

        miden_air_impl.new_fn("aux_width").arg_ref_self().ret("usize").line("AUX_WIDTH");
    }

    // add the build_aux_trace function if needed
    if ir.num_random_values > 0 {
        let build_aux_trace_func = miden_air_impl
            .new_fn("build_aux_trace")
            .arg_ref_self()
            .arg("_main", "&RowMajorMatrix<F>")
            .arg("_challenges", "&[EF]")
            .ret("Option<RowMajorMatrix<F>>");
        build_aux_trace_func.line("// Note: consider using Some(build_aux_trace_with_miden_vm::<F, EF>(_main, _challenges, module)) if you want to build the aux trace using Miden VM aux trace builders.");
        build_aux_trace_func.line("");
        build_aux_trace_func.line("let num_rows = _main.height();");
        build_aux_trace_func.line("let trace_length = num_rows * AUX_WIDTH;");
        build_aux_trace_func.line("let mut long_trace = EF::zero_vec(trace_length);");
        build_aux_trace_func.line("let mut trace = RowMajorMatrix::new(long_trace, AUX_WIDTH);");
        build_aux_trace_func.line("let (prefix, rows, suffix) = unsafe { trace.values.align_to_mut::<[EF; AUX_WIDTH]>() };");
        build_aux_trace_func.line("assert!(prefix.is_empty(), \"Alignment should match\");");
        build_aux_trace_func.line("assert!(suffix.is_empty(), \"Alignment should match\");");
        build_aux_trace_func.line("assert_eq!(rows.len(), num_rows);");
        build_aux_trace_func.line("// Initialize first row");
        build_aux_trace_func.line("let initial_values = Self::buses_initial_values::<F, EF>();");
        build_aux_trace_func.line("for j in 0..AUX_WIDTH {");
        build_aux_trace_func.line("    rows[0][j] = initial_values[j];");
        build_aux_trace_func.line("}");
        build_aux_trace_func.line("// Fill subsequent rows using direct access to the rows array");
        build_aux_trace_func.line("for i in 0..num_rows-1 {");
        build_aux_trace_func.line("    let i_next = (i + 1) % num_rows;");
        build_aux_trace_func.line("    let main_local = _main.row_slice(i).unwrap(); // i < height so unwrap should never fail.");
        build_aux_trace_func.line("    let main_next = _main.row_slice(i_next).unwrap(); // i_next < height so unwrap should never fail.");
        build_aux_trace_func.line("    let main = VerticalPair::new(");
        build_aux_trace_func.line("        RowMajorMatrixView::new_row(&*main_local),");
        build_aux_trace_func.line("        RowMajorMatrixView::new_row(&*main_next),");
        build_aux_trace_func.line("    );");
        build_aux_trace_func.line(format!("    let periodic_values: [_; NUM_PERIODIC_VALUES] = <{name} as MidenAir<F, EF>>::periodic_table(self).iter().map(|col| col[i % col.len()]).collect::<Vec<_>>().try_into().expect(\"Wrong number of periodic values\");"));
        build_aux_trace_func.line("    let prev_row = &rows[i];");
        build_aux_trace_func.line("    let next_row = Self::buses_transitions::<F, EF>(");
        build_aux_trace_func.line("        &main,");
        build_aux_trace_func.line("        _challenges,");
        build_aux_trace_func.line("        &periodic_values,");
        build_aux_trace_func.line("        prev_row,");
        build_aux_trace_func.line("    );");
        build_aux_trace_func.line("    for j in 0..AUX_WIDTH {");
        build_aux_trace_func.line("        rows[i+1][j] = next_row[j];");
        build_aux_trace_func.line("    }");
        build_aux_trace_func.line("}");
        build_aux_trace_func.line("let trace_f = trace.flatten_to_base();");
        build_aux_trace_func.line("Some(trace_f)");
    }

    // add the eval function
    let eval_func = miden_air_impl
        .new_fn("eval")
        .generic("AB")
        .bound("AB", "MidenAirBuilder<F = F>")
        .arg_ref_self()
        .arg("builder", "&mut AB");
    eval_func.line("let public_values: [_; NUM_PUBLIC_VALUES] = builder.public_values().try_into().expect(\"Wrong number of public values\");");
    eval_func.line("let periodic_values: [_; NUM_PERIODIC_VALUES] = builder.periodic_evals().try_into().expect(\"Wrong number of periodic values\");");

    eval_func.line("// Note: for now, we do not have any preprocessed values");
    eval_func.line("// let preprocessed = builder.preprocessed();");

    eval_func.line("let main = builder.main();");
    eval_func.line("let (main_current, main_next) = (");
    eval_func.line("    main.row_slice(0).unwrap(),");
    eval_func.line("    main.row_slice(1).unwrap(),");
    eval_func.line(");");

    // Only add aux if there are random values
    if ir.num_random_values > 0 {
        eval_func.line("let (&alpha, beta_challenges) = builder.permutation_randomness().split_first().expect(\"Wrong number of randomness\");");
        eval_func.line("let beta_challenges: [_; MAX_BETA_CHALLENGE_POWER] = beta_challenges.try_into().expect(\"Wrong number of randomness\");");
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

/// Updates the provided scope with aux trace generation utilities.
fn add_aux_trace_utils(scope: &mut Scope, ir: &Air, name: &str) {
    let aux_generation_impl = scope.new_impl(name);

    // add the bus_initial_values function
    let buses_initial_values_func = aux_generation_impl
        .new_fn("buses_initial_values")
        .generic("F")
        .generic("EF")
        .bound("F", "Field")
        .bound("EF", "ExtensionField<F>")
        .ret("Vec<EF>");
    buses_initial_values_func.line("vec![");
    for (_bus_id, value) in ir.buses_initial_values.iter() {
        let value_str = value.to_string(ir, ElemType::ExtFieldElem);

        buses_initial_values_func.line(format!("    {},", value_str));
    }
    buses_initial_values_func.line("]");

    // add the bus_transitions function
    let buses_transitions_func = aux_generation_impl
        .new_fn("buses_transitions")
        .generic("F")
        .generic("EF")
        .bound("F", "Field")
        .bound("EF", "ExtensionField<F>")
        .arg("main", "&VerticalPair<RowMajorMatrixView<F>, RowMajorMatrixView<F>>")
        .arg("challenges", "&[EF]")
        .arg("periodic_evals", "&[F]")
        .arg("aux_current", "&[EF]")
        .ret("Vec<EF>");

    buses_transitions_func.line("let (main_current, main_next) = (");
    buses_transitions_func.line("    main.row_slice(0).unwrap(),");
    buses_transitions_func.line("    main.row_slice(1).unwrap(),");
    buses_transitions_func.line(");");
    buses_transitions_func.line("let (&alpha, beta_challenges) = challenges.split_first().expect(\"Wrong number of randomness\");");
    buses_transitions_func.line("let beta_challenges: [_; MAX_BETA_CHALLENGE_POWER] = beta_challenges.try_into().expect(\"Wrong number of randomness\");");

    buses_transitions_func.line("let periodic_values: [_; NUM_PERIODIC_VALUES] = periodic_evals.try_into().expect(\"Wrong number of periodic values\");");

    buses_transitions_func.line("vec![");
    for (_bus_id, (numerator, denominator)) in ir.buses_transitions.iter() {
        let numerator_str = numerator.to_string(ir, ElemType::ExtFieldElem);

        let aux_next_value_str = if let Some(denom) = denominator {
            let denominator_str = denom.to_string(ir, ElemType::ExtFieldElem);
            format!("({}) * ({}).inverse()", numerator_str, denominator_str)
        } else {
            numerator_str
        };

        buses_transitions_func.line(format!("    {},", aux_next_value_str));
    }
    buses_transitions_func.line("]");
}
