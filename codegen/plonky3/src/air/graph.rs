use air_ir::{Air, NodeIndex, Operation, TraceAccess, Value};

// RUST STRING GENERATION FOR THE CONSTRAINT GRAPH
// ================================================================================================

/// Code generation trait for generating Rust code strings from IR types related to constraints and
/// the [AlgebraicGraph].
pub trait Codegen {
    fn to_string(&self, ir: &Air) -> String;
}

impl Codegen for TraceAccess {
    fn to_string(&self, _ir: &Air) -> String {
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
        format!("{frame}_{row_offset}")
    }
}

impl Codegen for NodeIndex {
    fn to_string(&self, ir: &Air) -> String {
        let op = ir.constraint_graph().node(self).op();
        op.to_string(ir)
    }
}

impl Codegen for Operation {
    fn to_string(&self, ir: &Air) -> String {
        match self {
            Operation::Value(value) => value.to_string(ir),
            Operation::Add(..) => binary_op_to_string(ir, self),
            Operation::Sub(..) => binary_op_to_string(ir, self),
            Operation::Mul(..) => binary_op_to_string(ir, self),
        }
    }
}

impl Codegen for Value {
    fn to_string(&self, ir: &Air) -> String {
        match self {
            Value::Constant(value) => format!("AB::Expr::from(AB::F::from_u64({value}))"),
            Value::TraceAccess(trace_access) => trace_access.to_string(ir),
            Value::PublicInput(air_ir::PublicInputAccess { name, index }) => {
                let get_public_input_offset = |name: &str| {
                    ir.public_inputs()
                        .take_while(|pi| pi.name() != name)
                        .map(|pi| pi.size())
                        .sum::<usize>()
                };
                format!("public_values[{}].into()", get_public_input_offset(name.as_str()) + index)
            },
            Value::PeriodicColumn(pc) => {
                let index =
                    ir.periodic_columns.iter().position(|(qid, _)| qid == &pc.name).unwrap();
                format!("periodic_values[{index}].into()")
            },
            _ => todo!(),
            /*Value::PublicInputTable(air_ir::PublicInputTableAccess {
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
fn binary_op_to_string(ir: &Air, op: &Operation) -> String {
    match op {
        Operation::Add(l_idx, r_idx) => {
            let lhs = l_idx.to_string(ir);
            let rhs = r_idx.to_string(ir);
            format!("{lhs} + {rhs}")
        },
        Operation::Sub(l_idx, r_idx) => {
            let lhs = l_idx.to_string(ir);
            let rhs = if ir.constraint_graph().node(r_idx).op().precedence() <= op.precedence() {
                format!("({})", r_idx.to_string(ir))
            } else {
                r_idx.to_string(ir)
            };
            format!("{lhs} - {rhs}")
        },
        Operation::Mul(l_idx, r_idx) => {
            let lhs = if ir.constraint_graph().node(l_idx).op().precedence() < op.precedence() {
                format!("({})", l_idx.to_string(ir))
            } else {
                l_idx.to_string(ir)
            };
            let rhs = if ir.constraint_graph().node(r_idx).op().precedence() < op.precedence() {
                format!("({})", r_idx.to_string(ir))
            } else {
                r_idx.to_string(ir)
            };
            format!("{lhs} * {rhs}")
        },
        _ => panic!("unsupported operation"),
    }
}
