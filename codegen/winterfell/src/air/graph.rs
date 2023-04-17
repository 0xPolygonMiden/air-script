use super::{
    AccessType, AirIR, ElemType, IntegrityConstraintDegree, NodeIndex, Operation, TraceAccess,
    Value,
};

// RUST STRING GENERATION FOR THE CONSTRAINT GRAPH
// ================================================================================================

/// Code generation trait for generating Rust code strings from IR types related to constraints and
/// the [AlgebraicGraph].
/// TODO: replace panics with errors
pub trait Codegen {
    fn to_string(&self, ir: &AirIR, elem_type: ElemType, trace_segment: u8) -> String;
}

impl Codegen for IntegrityConstraintDegree {
    fn to_string(&self, _ir: &AirIR, _elem_type: ElemType, _trace_segment: u8) -> String {
        if self.cycles().is_empty() {
            format!("TransitionConstraintDegree::new({})", self.base())
        } else {
            let cycles = self
                .cycles()
                .iter()
                .map(|cycle_len| cycle_len.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            format!(
                "TransitionConstraintDegree::with_cycles({}, vec![{}])",
                self.base(),
                cycles
            )
        }
    }
}

impl Codegen for TraceAccess {
    fn to_string(&self, _ir: &AirIR, _elem_type: ElemType, trace_segment: u8) -> String {
        let frame = if let 0 = self.trace_segment() {
            "main"
        } else {
            "aux"
        };
        let row_offset = match self.row_offset() {
            0 => {
                format!("current[{}]", self.col_idx())
            }
            1 => {
                format!("next[{}]", self.col_idx())
            }
            _ => panic!("Winterfell doesn't support row offsets greater than 1."),
        };
        if self.trace_segment() == 0 && self.trace_segment() != trace_segment {
            format!("E::from({frame}_{row_offset})")
        } else {
            format!("{frame}_{row_offset}")
        }
    }
}

impl Codegen for NodeIndex {
    fn to_string(&self, ir: &AirIR, elem_type: ElemType, trace_segment: u8) -> String {
        let op = ir.constraint_graph().node(self).op();
        op.to_string(ir, elem_type, trace_segment)
    }
}

impl Codegen for Operation {
    fn to_string(&self, ir: &AirIR, elem_type: ElemType, trace_segment: u8) -> String {
        match self {
            Operation::Value(value) => value.to_string(ir, elem_type, trace_segment),
            Operation::Add(_, _) => binary_op_to_string(ir, self, elem_type, trace_segment),
            Operation::Sub(_, _) => binary_op_to_string(ir, self, elem_type, trace_segment),
            Operation::Mul(_, _) => binary_op_to_string(ir, self, elem_type, trace_segment),
            // TODO: move this logic to a helper function
            Operation::Exp(l_idx, r_idx) => {
                let lhs = l_idx.to_string(ir, elem_type, trace_segment);
                let lhs = if is_leaf(l_idx, ir) {
                    lhs
                } else {
                    format!("({lhs})")
                };
                match r_idx {
                    0 => match elem_type {
                        // x^0 = 1
                        ElemType::Base => "Felt::ONE".to_string(),
                        ElemType::Ext => "E::ONE".to_string(),
                    },
                    1 => lhs, // x^1 = x
                    _ => match elem_type {
                        ElemType::Base => format!("{lhs}.exp(Felt::new({r_idx}))"),
                        ElemType::Ext => {
                            format!("{lhs}.exp(E::PositiveInteger::from({r_idx}_u64))")
                        }
                    },
                }
            }
        }
    }
}

impl Codegen for Value {
    fn to_string(&self, ir: &AirIR, elem_type: ElemType, trace_segment: u8) -> String {
        match self {
            // TODO: move constant handling to a helper function
            Value::InlineConstant(0) => match elem_type {
                ElemType::Base => "Felt::ZERO".to_string(),
                ElemType::Ext => "E::ZERO".to_string(),
            },
            Value::InlineConstant(1) => match elem_type {
                ElemType::Base => "Felt::ONE".to_string(),
                ElemType::Ext => "E::ONE".to_string(),
            },
            Value::InlineConstant(value) => match elem_type {
                ElemType::Base => format!("Felt::new({value})"),
                ElemType::Ext => format!("E::from({value}_u64)"),
            },
            Value::BoundConstant(symbol_access) => {
                let name = symbol_access.name().to_string();
                let access_type = symbol_access.access_type();
                let base_value = match access_type {
                    AccessType::Default => name,
                    AccessType::Vector(idx) => format!("{name}[{idx}]"),
                    AccessType::Matrix(row_idx, col_idx) => {
                        format!("{name}[{row_idx}][{col_idx}]",)
                    }
                    AccessType::Slice(_) => panic!("unsupported access type"),
                };
                match elem_type {
                    ElemType::Base => base_value,
                    ElemType::Ext => format!("E::from({base_value})"),
                }
            }
            Value::TraceElement(trace_access) => {
                trace_access.to_string(ir, elem_type, trace_segment)
            }
            Value::PeriodicColumn(col_idx, _) => {
                format!("periodic_values[{col_idx}]")
            }
            Value::PublicInput(ident, idx) => {
                format!("self.{ident}[{idx}]")
            }
            Value::RandomValue(idx) => {
                format!("aux_rand_elements.get_segment_elements(0)[{idx}]")
            }
        }
    }
}

/// Returns true if the operation at the specified node index is a leaf node in the constraint graph.
fn is_leaf(idx: &NodeIndex, ir: &AirIR) -> bool {
    !matches!(
        ir.constraint_graph().node(idx).op(),
        Operation::Add(_, _) | Operation::Sub(_, _) | Operation::Mul(_, _) | Operation::Exp(_, _)
    )
}

/// Returns a string representation of a binary operation.
fn binary_op_to_string(
    ir: &AirIR,
    op: &Operation,
    elem_type: ElemType,
    trace_segment: u8,
) -> String {
    match op {
        Operation::Add(l_idx, r_idx) => {
            let lhs = l_idx.to_string(ir, elem_type, trace_segment);
            let rhs = r_idx.to_string(ir, elem_type, trace_segment);
            format!("{lhs} + {rhs}")
        }
        Operation::Sub(l_idx, r_idx) => {
            let lhs = l_idx.to_string(ir, elem_type, trace_segment);
            let rhs = if ir.constraint_graph().node(r_idx).op().precedence() <= op.precedence() {
                format!("({})", r_idx.to_string(ir, elem_type, trace_segment))
            } else {
                r_idx.to_string(ir, elem_type, trace_segment)
            };
            format!("{lhs} - {rhs}")
        }
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
        }
        _ => panic!("unsupported operation"),
    }
}
