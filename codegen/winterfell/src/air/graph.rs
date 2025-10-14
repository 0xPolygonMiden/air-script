use air_ir::{
    Air, IntegrityConstraintDegree, NodeIndex, Operation, TraceAccess, TraceSegmentId, Value,
};

use super::ElemType;

// RUST STRING GENERATION FOR THE CONSTRAINT GRAPH
// ================================================================================================

/// Code generation trait for generating Rust code strings from IR types related to constraints and
/// the [AlgebraicGraph].
/// TODO: replace panics with errors
pub trait Codegen {
    fn to_string(&self, ir: &Air, elem_type: ElemType, trace_segment: TraceSegmentId) -> String;
}

impl Codegen for IntegrityConstraintDegree {
    fn to_string(&self, _ir: &Air, _elem_type: ElemType, _trace_segment: TraceSegmentId) -> String {
        if self.cycles().is_empty() {
            format!("TransitionConstraintDegree::new({})", self.base())
        } else {
            let cycles = self
                .cycles()
                .iter()
                .map(|cycle_len| cycle_len.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            format!("TransitionConstraintDegree::with_cycles({}, vec![{}])", self.base(), cycles)
        }
    }
}

impl Codegen for TraceAccess {
    fn to_string(&self, _ir: &Air, _elem_type: ElemType, trace_segment: TraceSegmentId) -> String {
        let frame = self.segment.to_string();
        let row_offset = match self.row_offset {
            0 => {
                format!("current[{}]", self.column)
            },
            1 => {
                format!("next[{}]", self.column)
            },
            _ => panic!("Winterfell doesn't support row offsets greater than 1."),
        };
        if self.segment == TraceSegmentId::Main && self.segment != trace_segment {
            format!("E::from({frame}_{row_offset})")
        } else {
            format!("{frame}_{row_offset}")
        }
    }
}

impl Codegen for NodeIndex {
    fn to_string(&self, ir: &Air, elem_type: ElemType, trace_segment: TraceSegmentId) -> String {
        let op = ir.constraint_graph().node(self).op();
        op.to_string(ir, elem_type, trace_segment)
    }
}

impl Codegen for Operation {
    fn to_string(&self, ir: &Air, elem_type: ElemType, trace_segment: TraceSegmentId) -> String {
        match self {
            Operation::Value(value) => value.to_string(ir, elem_type, trace_segment),
            Operation::Add(..) => binary_op_to_string(ir, self, elem_type, trace_segment),
            Operation::Sub(..) => binary_op_to_string(ir, self, elem_type, trace_segment),
            Operation::Mul(..) => binary_op_to_string(ir, self, elem_type, trace_segment),
        }
    }
}

impl Codegen for Value {
    fn to_string(&self, ir: &Air, elem_type: ElemType, trace_segment: TraceSegmentId) -> String {
        match self {
            // TODO: move constant handling to a helper function
            Value::Constant(0) => match elem_type {
                ElemType::Base => "Felt::ZERO".to_string(),
                ElemType::Ext => "E::ZERO".to_string(),
            },
            Value::Constant(1) => match elem_type {
                ElemType::Base => "Felt::ONE".to_string(),
                ElemType::Ext => "E::ONE".to_string(),
            },
            Value::Constant(value) => match elem_type {
                ElemType::Base => format!("Felt::new({value})"),
                ElemType::Ext => format!("E::from(Felt::new({value}_u64))"),
            },
            Value::TraceAccess(trace_access) => {
                trace_access.to_string(ir, elem_type, trace_segment)
            },
            Value::PeriodicColumn(pc) => {
                let index =
                    ir.periodic_columns.iter().position(|(qid, _)| qid == &pc.name).unwrap();
                format!("periodic_values[{index}]")
            },
            Value::PublicInput(air_ir::PublicInputAccess { name, index }) => {
                format!("self.{name}[{index}]")
            },
            Value::PublicInputTable(air_ir::PublicInputTableAccess {
                table_name,
                bus_type,
                num_cols: _,
            }) => {
                format!("reduced_{table_name}_{bus_type}")
            },
            Value::RandomValue(idx) => {
                format!("aux_rand_elements.rand_elements()[{idx}]")
            },
        }
    }
}

/// Returns a string representation of a binary operation.
fn binary_op_to_string(
    ir: &Air,
    op: &Operation,
    elem_type: ElemType,
    trace_segment: TraceSegmentId,
) -> String {
    match op {
        Operation::Add(l_idx, r_idx) => {
            let lhs = l_idx.to_string(ir, elem_type, trace_segment);
            let rhs = r_idx.to_string(ir, elem_type, trace_segment);
            format!("{lhs} + {rhs}")
        },
        Operation::Sub(l_idx, r_idx) => {
            let lhs = l_idx.to_string(ir, elem_type, trace_segment);
            let rhs = if ir.constraint_graph().node(r_idx).op().precedence() <= op.precedence() {
                format!("({})", r_idx.to_string(ir, elem_type, trace_segment))
            } else {
                r_idx.to_string(ir, elem_type, trace_segment)
            };
            format!("{lhs} - {rhs}")
        },
        Operation::Mul(l_idx, r_idx) => {
            let lhs = if ir.constraint_graph().node(l_idx).op().precedence() < op.precedence() {
                format!("({})", l_idx.to_string(ir, elem_type, trace_segment))
            } else {
                l_idx.to_string(ir, elem_type, trace_segment)
            };
            let rhs = if ir.constraint_graph().node(r_idx).op().precedence() < op.precedence() {
                format!("({})", r_idx.to_string(ir, elem_type, trace_segment))
            } else {
                r_idx.to_string(ir, elem_type, trace_segment)
            };
            format!("{lhs} * {rhs}")
        },
        _ => panic!("unsupported operation"),
    }
}

// COMMON-SUBEXPRESSION ELIMINATION & CONVERSION HOISTING EMITTER
// ================================================================================================
// The functions below generate an expression string along with a list of temporary bindings which
// should be emitted before using the expression. This reduces duplicate multiplications produced by
// exponentiation lowering, and factors repeated extension field conversions (E::from(...)).

use std::collections::HashMap;

/// Returns a tuple of (temporary let bindings, final expression string) for the provided node.
pub(super) fn to_string_with_temps(
    ir: &Air,
    node_index: &NodeIndex,
    trace_segment: TraceSegmentId,
) -> (Vec<String>, String) {
    let mut use_counts: HashMap<NodeIndex, usize> = HashMap::new();
    collect_use_counts(ir, node_index, &mut use_counts);

    let mut cse_temps: HashMap<NodeIndex, String> = HashMap::new();
    let mut conv_seen: HashMap<String, usize> = HashMap::new();
    let mut conv_temps: HashMap<String, String> = HashMap::new();
    let mut temps: Vec<String> = Vec::new();
    let mut temp_counter: usize = 0;

    let expr = emit_node(
        ir,
        node_index,
        trace_segment,
        &use_counts,
        &mut cse_temps,
        &mut conv_seen,
        &mut conv_temps,
        &mut temps,
        &mut temp_counter,
    );

    (temps, expr)
}

fn collect_use_counts(
    ir: &Air,
    node_index: &NodeIndex,
    use_counts: &mut HashMap<NodeIndex, usize>,
) {
    *use_counts.entry(*node_index).or_insert(0) += 1;
    match ir.constraint_graph().node(node_index).op() {
        Operation::Add(l, r) | Operation::Sub(l, r) | Operation::Mul(l, r) => {
            collect_use_counts(ir, l, use_counts);
            collect_use_counts(ir, r, use_counts);
        },
        Operation::Value(_) => {},
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_node(
    ir: &Air,
    node_index: &NodeIndex,
    trace_segment: TraceSegmentId,
    use_counts: &HashMap<NodeIndex, usize>,
    cse_temps: &mut HashMap<NodeIndex, String>,
    conv_seen: &mut HashMap<String, usize>,
    conv_temps: &mut HashMap<String, String>,
    temps: &mut Vec<String>,
    temp_counter: &mut usize,
) -> String {
    // If this node is a candidate for CSE and has been assigned a temp, return it.
    if let Some(name) = cse_temps.get(node_index) {
        return name.clone();
    }

    match ir.constraint_graph().node(node_index).op() {
        Operation::Value(v) => {
            // Delegate to existing value printer, then hoist repeated E::from conversions.
            let s = v.to_string(ir, ElemType::Ext, trace_segment);
            if s.starts_with("E::from(") {
                let count = conv_seen.entry(s.clone()).or_insert(0);
                *count += 1;
                if *count >= 2 {
                    if let Some(name) = conv_temps.get(&s) {
                        return name.clone();
                    }
                    let name = format!("t{}", *temp_counter);
                    *temp_counter += 1;
                    temps.push(format!("let {name} = {s};"));
                    conv_temps.insert(s, name.clone());
                    return name;
                }
            }
            s
        },
        Operation::Add(l, r) | Operation::Sub(l, r) | Operation::Mul(l, r) => {
            let lhs_str = emit_child(
                ir,
                l,
                node_index,
                trace_segment,
                use_counts,
                cse_temps,
                conv_seen,
                conv_temps,
                temps,
                temp_counter,
            );
            let rhs_str = emit_child(
                ir,
                r,
                node_index,
                trace_segment,
                use_counts,
                cse_temps,
                conv_seen,
                conv_temps,
                temps,
                temp_counter,
            );

            let op = ir.constraint_graph().node(node_index).op();
            let rendered = match op {
                Operation::Add(..) => format!("{lhs_str} + {rhs_str}"),
                Operation::Sub(..) => format!("{lhs_str} - {rhs_str}"),
                Operation::Mul(..) => format!("{lhs_str} * {rhs_str}"),
                _ => unreachable!(),
            };

            if use_counts.get(node_index).copied().unwrap_or(1) > 1 {
                let name = format!("t{}", *temp_counter);
                *temp_counter += 1;
                temps.push(format!("let {name} = {rendered};"));
                cse_temps.insert(*node_index, name.clone());
                name
            } else {
                rendered
            }
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_child(
    ir: &Air,
    child: &NodeIndex,
    parent: &NodeIndex,
    trace_segment: TraceSegmentId,
    use_counts: &HashMap<NodeIndex, usize>,
    cse_temps: &mut HashMap<NodeIndex, String>,
    conv_seen: &mut HashMap<String, usize>,
    conv_temps: &mut HashMap<String, String>,
    temps: &mut Vec<String>,
    temp_counter: &mut usize,
) -> String {
    let needs_paren = ir.constraint_graph().node(child).op().precedence()
        < ir.constraint_graph().node(parent).op().precedence();
    let s = emit_node(
        ir,
        child,
        trace_segment,
        use_counts,
        cse_temps,
        conv_seen,
        conv_temps,
        temps,
        temp_counter,
    );
    if needs_paren { format!("({s})") } else { s }
}
