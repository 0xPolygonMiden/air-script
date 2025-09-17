use air_ir::{
    Air, IntegrityConstraintDegree, NodeIndex, Operation, TraceAccess, TraceSegmentId, Value,
};

// RUST STRING GENERATION FOR THE CONSTRAINT GRAPH
// ================================================================================================

/// Code generation trait for generating Rust code strings from IR types related to constraints and
/// the [AlgebraicGraph].
pub trait Codegen {
    fn to_string(&self, ir: &Air, trace_segment: TraceSegmentId) -> String;
}

impl Codegen for IntegrityConstraintDegree {
    fn to_string(&self, _ir: &Air, _trace_segment: TraceSegmentId) -> String {
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
    fn to_string(&self, _ir: &Air, trace_segment: TraceSegmentId) -> String {
        let frame = self.segment.to_string();
        let row_offset = match self.row_offset {
            0 => {
                format!("current[{}]", self.column)
            },
            1 => {
                format!("next[{}]", self.column)
            },
            _ => panic!("Plonky3 doesn't support row offsets greater than 1."),
        };
        if self.segment == TraceSegmentId::Main && self.segment != trace_segment {
            format!("E::from({frame}_{row_offset})")
        } else {
            format!("{frame}_{row_offset}")
        }
    }
}

impl Codegen for NodeIndex {
    fn to_string(&self, ir: &Air, trace_segment: TraceSegmentId) -> String {
        let op = ir.constraint_graph().node(self).op();
        op.to_string(ir, trace_segment)
    }
}

impl Codegen for Operation {
    fn to_string(&self, ir: &Air, trace_segment: TraceSegmentId) -> String {
        match self {
            Operation::Value(value) => value.to_string(ir, trace_segment),
            Operation::Add(..) => binary_op_to_string(ir, self, trace_segment),
            Operation::Sub(..) => binary_op_to_string(ir, self, trace_segment),
            Operation::Mul(..) => binary_op_to_string(ir, self, trace_segment),
        }
    }
}

impl Codegen for Value {
    fn to_string(&self, ir: &Air, trace_segment: TraceSegmentId) -> String {
        match self {
            Value::Constant(value) => format!("AB::Expr::from(AB::F::from_u64({value}))"),
            Value::TraceAccess(trace_access) => trace_access.to_string(ir, trace_segment),
            Value::PublicInput(air_ir::PublicInputAccess { name, index }) => {
                let get_public_input_offset = |name: &str| {
                    ir.public_inputs()
                        .take_while(|pi| pi.name() != name)
                        .map(|pi| pi.size())
                        .sum::<usize>()
                };
                format!("public_values[{}].into()", get_public_input_offset(name.as_str()) + index)
            },
            _ => todo!(),
            /*Value::PeriodicColumn(pc) => {
                let index =
                    ir.periodic_columns.iter().position(|(qid, _)| qid == &pc.name).unwrap();
                format!("periodic_values[{index}]")
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
            },*/
        }
    }
}

/// Returns a string representation of a binary operation.
fn binary_op_to_string(ir: &Air, op: &Operation, trace_segment: TraceSegmentId) -> String {
    match op {
        Operation::Add(l_idx, r_idx) => {
            let lhs = l_idx.to_string(ir, trace_segment);
            let rhs = r_idx.to_string(ir, trace_segment);
            format!("{lhs} + {rhs}")
        },
        Operation::Sub(l_idx, r_idx) => {
            let lhs = l_idx.to_string(ir, trace_segment);
            let rhs = if ir.constraint_graph().node(r_idx).op().precedence() <= op.precedence() {
                format!("({})", r_idx.to_string(ir, trace_segment))
            } else {
                r_idx.to_string(ir, trace_segment)
            };
            format!("{lhs} - {rhs}")
        },
        Operation::Mul(l_idx, r_idx) => {
            let lhs = if ir.constraint_graph().node(l_idx).op().precedence() < op.precedence() {
                format!("({})", l_idx.to_string(ir, trace_segment))
            } else {
                l_idx.to_string(ir, trace_segment)
            };
            let rhs = if ir.constraint_graph().node(r_idx).op().precedence() < op.precedence() {
                format!("({})", r_idx.to_string(ir, trace_segment))
            } else {
                r_idx.to_string(ir, trace_segment)
            };
            format!("{lhs} * {rhs}")
        },
        _ => panic!("unsupported operation"),
    }
}
