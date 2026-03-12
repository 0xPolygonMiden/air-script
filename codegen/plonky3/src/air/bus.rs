//! Bus-related codegen: aux trace building (multiset and logup), request/response helpers.
//! Naming follows Miden VM AuxColumnBuilder (request = denominator, response = numerator).

use air_ir::{Air, BusOpKind, BusType, NodeIndex, Operation};

use super::Scope;
use crate::air::{ElemType, graph::Codegen};

// ---------------------------------------------------------------------------
// Helpers for generating factor/latch expressions and product/join strings
// ---------------------------------------------------------------------------

/// Factor expression for a bus op: challenges[0] + challenges[1]*col0 + challenges[2]*col1 + ...
/// Matches IR: RandomValue(0) + col0*RandomValue(1) + col1*RandomValue(2) + ...
fn bus_op_factor_expr(ir: &Air, columns: &[NodeIndex]) -> String {
    if columns.is_empty() {
        return "challenges[0]".to_string();
    }
    let mut parts = vec!["challenges[0]".to_string()];
    let mul_prec = Operation::Mul(NodeIndex::default(), NodeIndex::default()).precedence();
    for (i, &col) in columns.iter().enumerate() {
        let op = ir.constraint_graph().node(&col).op();
        let col_str = col.to_string(ir, ElemType::ExtFieldElem);
        let term = if op.precedence() < mul_prec {
            format!("challenges[{}] * ({})", i + 1, col_str)
        } else {
            format!("challenges[{}] * {}", i + 1, col_str)
        };
        parts.push(term);
    }
    parts.join(" + ")
}

/// Latch expression string, with parens if needed for precedence in multiplications.
fn latch_expr_str(ir: &Air, latch: NodeIndex) -> String {
    let s = latch.to_string(ir, ElemType::ExtFieldElem);
    let latch_op = ir.constraint_graph().node(&latch).op();
    if latch_op.precedence()
        < Operation::Mul(NodeIndex::default(), NodeIndex::default()).precedence()
    {
        format!("({})", s)
    } else {
        s
    }
}

/// Product of factor strings, each wrapped in parens. If `exclude` is Some(i), omit factor i.
fn product_of_factor_strs(factor_strs: &[String], exclude: Option<usize>) -> String {
    let iter = factor_strs
        .iter()
        .enumerate()
        .filter(|(i, _)| exclude != Some(*i))
        .map(|(_, s)| format!("({})", s));
    match iter.reduce(|a, b| format!("{} * {}", a, b)) {
        Some(s) => s,
        None => "EF::ONE".to_string(),
    }
}

/// Join parts with `join_with`, or return `empty` if no parts, or single part as-is.
fn join_with_default(parts: &[String], empty: &str, join_with: &str) -> String {
    match parts {
        [] => empty.to_string(),
        [one] => one.clone(),
        _ => parts.join(join_with),
    }
}

const BUS_HELPER_PREAMBLE: [&str; 5] = [
    "let (main_current, _main_next) = (",
    "    main.row_slice(0).unwrap(),",
    "    main.row_slice(1).unwrap(),",
    ");",
    "let _periodic_values: [_; NUM_PERIODIC_VALUES] = periodic_evals.try_into().expect(\"Wrong number of periodic values\");",
];

// ---------------------------------------------------------------------------
// build_aux_trace body: fill multiset and logup columns
// ---------------------------------------------------------------------------

/// Appends the bus-filling portion of `build_aux_trace` (multiset + logup columns).
/// Caller adds preamble (num_rows, trace, initial_values) and trailing (trace_f, Some(trace_f)).
pub(super) fn add_build_aux_trace_body<F>(add_line: &mut F, ir: &Air, name: &str)
where
    F: FnMut(&str),
{
    let multiset_indices: Vec<usize> = ir
        .buses
        .iter()
        .enumerate()
        .filter(|(_, (_, bus))| bus.bus_type == BusType::Multiset)
        .map(|(index, _)| index)
        .collect();
    let logup_indices: Vec<usize> = ir
        .buses
        .iter()
        .enumerate()
        .filter(|(_, (_, bus))| bus.bus_type == BusType::Logup)
        .map(|(index, _)| index)
        .collect();

    add_line(&format!(
        "let multiset_indices: Vec<usize> = vec![{}];",
        multiset_indices.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ")
    ));
    add_line(&format!(
        "let logup_indices: Vec<usize> = vec![{}];",
        logup_indices.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ")
    ));
    add_line("");

    add_line(
        "// Multiset columns: same pattern as Miden AuxColumnBuilder::build_aux_column (request/response, batch inversion).",
    );
    for bus_index in &multiset_indices {
        add_line(&format!("// Fill multiset column {bus_index}"));
        add_line("let mut requests: Vec<EF> = unsafe { uninit_vector(num_rows) };");
        add_line("requests[0] = EF::ONE;");
        add_line("let mut responses_prod: Vec<EF> = unsafe { uninit_vector(num_rows) };");
        add_line("responses_prod[0] = EF::ONE;");
        add_line("let mut requests_running_prod = requests[0];");
        add_line(
            "// Product of all requests to be inverted, used to compute inverses of requests. (Miden utils.rs build_aux_column)",
        );
        add_line("for i in 0..num_rows - 1 {");
        add_line("    let i_next = (i + 1) % num_rows;");
        add_line(
            "    let main_local = _main.row_slice(i).unwrap(); // i < height so unwrap should never fail.",
        );
        add_line(
            "    let main_next = _main.row_slice(i_next).unwrap(); // i_next < height so unwrap should never fail.",
        );
        add_line("    let main = VerticalPair::new(");
        add_line("        RowMajorMatrixView::new_row(&*main_local),");
        add_line("        RowMajorMatrixView::new_row(&*main_next),");
        add_line("    );");
        add_line(&format!(
            "    let periodic_values: [_; NUM_PERIODIC_VALUES] = <{name} as MidenAir<F, EF>>::periodic_table(self).iter().map(|col| col[i % col.len()]).collect::<Vec<_>>().try_into().expect(\"Wrong number of periodic values\");"
        ));
        add_line("");
        add_line(&format!(
            "    let response = Self::bus_{bus_index}_multiset_responses_at::<F, EF>(&main, _challenges, &periodic_values);"
        ));
        add_line("    responses_prod[i + 1] = responses_prod[i] * response;");
        add_line(&format!(
            "    let request = Self::bus_{bus_index}_multiset_requests_at::<F, EF>(&main, _challenges, &periodic_values);"
        ));
        add_line("    requests[i + 1] = request;");
        add_line("    requests_running_prod *= request;");
        add_line("}");
        add_line("");
        add_line(
            "// Use batch-inversion method to compute running product of `response[i]/request[i]`.",
        );
        add_line("for i in 0..num_rows {");
        add_line(&format!("    rows[i][{bus_index}] = responses_prod[i];"));
        add_line("}");
        add_line("let mut requests_running_divisor = requests_running_prod.inverse();");
        add_line("for i in (0..num_rows).rev() {");
        add_line(&format!("    rows[i][{bus_index}] *= requests_running_divisor;"));
        add_line("    requests_running_divisor *= requests[i];");
        add_line("}");
        add_line("");
    }

    add_line("// Logup columns: denominator/product over factors, and per-row term contributing");
    add_line("// to the running sum (LogUp). Batch-invert denominators then build running sum.");
    add_line("let n = num_rows - 1;");
    for bus_index in &logup_indices {
        add_line("let mut denominators: Vec<EF> = unsafe { uninit_vector(n) };");
        add_line("let mut terms: Vec<EF> = unsafe { uninit_vector(n) };");
        add_line(
            "// After backward pass: inv_den_suffix_prod[i] = 1 / (denominator[i] * .. * denominator[n-1]) for batch inversion.",
        );
        add_line("let mut inv_den_suffix_prod: Vec<EF> = unsafe { uninit_vector(n) };");
        add_line("let mut acc = EF::ONE;");
        add_line("for i in 0..n {");
        add_line("    let row0 = _main.row_slice(i).unwrap();");
        add_line("    let row1 = _main.row_slice(i + 1).unwrap();");
        add_line("    let main = VerticalPair::new(");
        add_line("        RowMajorMatrixView::new_row(&*row0),");
        add_line("        RowMajorMatrixView::new_row(&*row1),");
        add_line("    );");
        add_line(&format!(
            "    let periodic_values: [_; NUM_PERIODIC_VALUES] = <{name} as MidenAir<F, EF>>::periodic_table(self).iter().map(|col| col[i % col.len()]).collect::<Vec<_>>().try_into().expect(\"Wrong number of periodic values\");"
        ));
        add_line(&format!(
            "    let denominator = Self::bus_{bus_index}_logup_denominator_at::<F, EF>(&main, _challenges, &periodic_values);"
        ));
        add_line(&format!(
            "    let term = Self::bus_{bus_index}_logup_term_at::<F, EF>(&main, _challenges, &periodic_values);"
        ));
        add_line("    denominators[i] = denominator;");
        add_line("    terms[i] = term;");
        add_line("    inv_den_suffix_prod[i] = acc;");
        add_line("    acc *= denominator;");
        add_line("}");
        add_line("acc = acc.inverse();");
        add_line("for i in (0..n).rev() {");
        add_line("    inv_den_suffix_prod[i] *= acc;");
        add_line("    acc *= denominators[i];");
        add_line("}");
        add_line(&format!("rows[0][{bus_index}] = initial_values[{bus_index}];"));
        add_line("for i in 0..n {");
        add_line(&format!(
            "    rows[i + 1][{bus_index}] = rows[i][{bus_index}] + terms[i] * inv_den_suffix_prod[i];"
        ));
        add_line("}");
        add_line("");
    }
}

// ---------------------------------------------------------------------------
// Aux trace utils impl: buses_initial_values, multiset/logup request/response fns
// ---------------------------------------------------------------------------

/// Adds the aux trace utils impl block: buses_initial_values and per-bus request/response helpers.
pub(super) fn add_aux_trace_utils(scope: &mut Scope, ir: &Air, name: &str) {
    let aux_generation_impl = scope.new_impl(name);

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

    // Multiset: request/response match Miden AuxColumnBuilder (get_requests_at = denominator,
    // get_responses_at = numerator).
    for (request_uses_remove, func_suffix) in [(true, "requests_at"), (false, "responses_at")] {
        let bus_op_kind = if request_uses_remove {
            BusOpKind::Remove
        } else {
            BusOpKind::Insert
        };
        for (bus_index, (_bus_ident, bus)) in ir.buses.iter().enumerate() {
            if bus.bus_type != BusType::Multiset {
                continue;
            }

            let func_name = format!("bus_{}_multiset_{}", bus_index, func_suffix);
            let func_def = aux_generation_impl
                .new_fn(&func_name)
                .generic("F")
                .generic("EF")
                .bound("F", "Field")
                .bound("EF", "ExtensionField<F>")
                .arg("main", "&VerticalPair<RowMajorMatrixView<F>, RowMajorMatrixView<F>>")
                .arg("challenges", "&[EF]")
                .arg("periodic_evals", "&[F]")
                .ret("EF");
            for line in BUS_HELPER_PREAMBLE {
                func_def.line(line);
            }
            // Each term is (factor * latch + (1 - latch)); wrap in parens only when joining
            // multiple.
            let term_strs: Vec<String> = bus
                .bus_ops
                .iter()
                .filter(|op| op.op_kind == bus_op_kind)
                .map(|bus_op| {
                    let factor_str = bus_op_factor_expr(ir, &bus_op.columns);
                    let latch_str = latch_expr_str(ir, bus_op.latch);
                    format!("({}) * {} + (EF::ONE - {})", factor_str, latch_str, latch_str)
                })
                .collect();
            let result_str = match term_strs.as_slice() {
                [] => "EF::ONE".to_string(),
                [one] => one.clone(),
                _ => term_strs.iter().map(|t| format!("({})", t)).collect::<Vec<_>>().join(" * "),
            };
            func_def.line(format!("let result = {};", result_str));
            func_def.line("if result == EF::ZERO {");
            func_def.line("    return EF::ONE;");
            func_def.line("}");
            func_def.line("result");
        }
    }

    // Logup: use denominator / term helpers (product over factors, and per-row term contributing
    // to the running sum), rather than multiset-style request/response.
    for (bus_index, (_bus_ident, bus)) in ir.buses.iter().enumerate() {
        if bus.bus_type != BusType::Logup {
            continue;
        }

        let factor_strs: Vec<String> =
            bus.bus_ops.iter().map(|op| bus_op_factor_expr(ir, &op.columns)).collect();

        {
            // Denominator: product over all factors for this row.
            let func_def = aux_generation_impl
                .new_fn(&format!("bus_{}_logup_denominator_at", bus_index))
                .generic("F")
                .generic("EF")
                .bound("F", "Field")
                .bound("EF", "ExtensionField<F>")
                .arg("main", "&VerticalPair<RowMajorMatrixView<F>, RowMajorMatrixView<F>>")
                .arg("challenges", "&[EF]")
                .arg("periodic_evals", "&[F]")
                .ret("EF");
            for line in BUS_HELPER_PREAMBLE {
                func_def.line(line);
            }
            func_def.line(product_of_factor_strs(&factor_strs, None));
        }

        {
            // Numerator term: net contribution (added − removed) to the LogUp running sum.
            let func_def = aux_generation_impl
                .new_fn(&format!("bus_{}_logup_term_at", bus_index))
                .generic("F")
                .generic("EF")
                .bound("F", "Field")
                .bound("EF", "ExtensionField<F>")
                .arg("main", "&VerticalPair<RowMajorMatrixView<F>, RowMajorMatrixView<F>>")
                .arg("challenges", "&[EF]")
                .arg("periodic_evals", "&[F]")
                .ret("EF");
            for line in BUS_HELPER_PREAMBLE {
                func_def.line(line);
            }
            let (mut terms_added, mut terms_removed) = (Vec::new(), Vec::new());
            for (op_index, bus_op) in bus.bus_ops.iter().enumerate() {
                let term_str = format!(
                    "{} * {}",
                    latch_expr_str(ir, bus_op.latch),
                    product_of_factor_strs(&factor_strs, Some(op_index))
                );
                match bus_op.op_kind {
                    BusOpKind::Insert => terms_added.push(term_str),
                    BusOpKind::Remove => terms_removed.push(term_str),
                }
            }
            func_def.line(format!(
                "let terms_added = {};",
                join_with_default(&terms_added, "EF::ZERO", " + ")
            ));
            func_def.line(format!(
                "let terms_removed = {};",
                join_with_default(&terms_removed, "EF::ZERO", " + ")
            ));
            func_def.line("terms_added - terms_removed");
        }
    }
}
