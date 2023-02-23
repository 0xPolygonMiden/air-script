use ir::IndexedTraceAccess;

use super::{AirIR, ConstantValue, ElemType, IntegrityConstraintDegree, NodeIndex, Operation};

// RUST STRING GENERATION FOR THE CONSTRAINT GRAPH
// ================================================================================================

/// Code generation trait for generating Rust code strings from IR types related to constraints and
/// the [AlgebraicGraph].
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

impl Codegen for IndexedTraceAccess {
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
            Operation::Constant(ConstantValue::Inline(value)) => match elem_type {
                ElemType::Base => format!("Felt::new({value})"),
                ElemType::Ext => format!("E::from({value}_u64)"),
            },
            Operation::Constant(ConstantValue::Scalar(ident)) => match elem_type {
                ElemType::Base => ident.to_string(),
                ElemType::Ext => format!("E::from({ident})"),
            },
            Operation::Constant(ConstantValue::Vector(vector_access)) => match elem_type {
                ElemType::Base => format!("{}[{}]", vector_access.name(), vector_access.idx()),
                ElemType::Ext => {
                    format!("E::from({}[{}])", vector_access.name(), vector_access.idx())
                }
            },
            Operation::Constant(ConstantValue::Matrix(matrix_access)) => match elem_type {
                ElemType::Base => format!(
                    "{}[{}][{}]",
                    matrix_access.name(),
                    matrix_access.row_idx(),
                    matrix_access.col_idx()
                ),
                ElemType::Ext => format!(
                    "E::from({}[{}][{}])",
                    matrix_access.name(),
                    matrix_access.row_idx(),
                    matrix_access.col_idx()
                ),
            },
            Operation::TraceElement(trace_access) => {
                trace_access.to_string(ir, elem_type, trace_segment)
            }
            Operation::PeriodicColumn(col_idx, _) => {
                format!("periodic_values[{col_idx}]")
            }
            Operation::PublicInput(ident, idx) => {
                format!("self.{ident}[{idx}]")
            }
            Operation::RandomValue(idx) => {
                format!("aux_rand_elements.get_segment_elements(0)[{idx}]")
            }
            Operation::Add(_, _) => binary_op_to_string(ir, self, elem_type, trace_segment),
            Operation::Sub(_, _) => binary_op_to_string(ir, self, elem_type, trace_segment),
            Operation::Mul(_, _) => binary_op_to_string(ir, self, elem_type, trace_segment),
            Operation::Exp(l_idx, r_idx) => {
                let lhs = l_idx.to_string(ir, elem_type, trace_segment);
                let lhs = if is_leaf(l_idx, ir) {
                    lhs
                } else {
                    format!("({lhs})")
                };
                match elem_type {
                    ElemType::Base => format!("{lhs}.exp(Felt::new({r_idx}))"),
                    ElemType::Ext => format!("{lhs}.exp(E::PositiveInteger::from({r_idx}_u64))"),
                }
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
