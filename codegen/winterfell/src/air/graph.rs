use super::{AirIR, ConstantValue, ElemType, IntegrityConstraintDegree, NodeIndex, Operation};

// RUST STRING GENERATION FOR THE CONSTRAINT GRAPH
// ================================================================================================

/// Code generation trait for generating Rust code strings from IR types related to constraints and
/// the [AlgebraicGraph].
pub trait Codegen {
    fn to_string(&self, ir: &AirIR, elem_type: ElemType) -> String;
}

impl Codegen for IntegrityConstraintDegree {
    fn to_string(&self, _ir: &AirIR, _elem_type: ElemType) -> String {
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

impl Codegen for NodeIndex {
    fn to_string(&self, ir: &AirIR, elem_type: ElemType) -> String {
        let op = ir.constraint_graph().node(self).op();
        op.to_string(ir, elem_type)
    }
}

impl Codegen for Operation {
    // TODO: Only add parentheses in Add and Mul if the expression is an arithmetic operation.
    fn to_string(&self, ir: &AirIR, elem_type: ElemType) -> String {
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
            Operation::TraceElement(trace_access) => match trace_access.row_offset() {
                0 => {
                    format!("current[{}]", trace_access.col_idx())
                }
                1 => {
                    format!("next[{}]", trace_access.col_idx())
                }
                _ => panic!("Winterfell doesn't support row offsets greater than 1."),
            },
            Operation::PeriodicColumn(col_idx, _) => {
                format!("periodic_values[{col_idx}]")
            }
            Operation::RandomValue(idx) => {
                format!("aux_rand_elements.get_segment_elements(0)[{idx}]")
            }
            Operation::Add(l_idx, r_idx) => {
                let lhs = l_idx.to_string(ir, elem_type);
                let rhs = r_idx.to_string(ir, elem_type);

                format!("{lhs} + {rhs}")
            }
            Operation::Sub(l_idx, r_idx) => {
                let lhs = l_idx.to_string(ir, elem_type);
                let rhs = r_idx.to_string(ir, elem_type);

                format!("{lhs} - ({rhs})")
            }
            Operation::Mul(l_idx, r_idx) => {
                let lhs = l_idx.to_string(ir, elem_type);
                let rhs = r_idx.to_string(ir, elem_type);
                format!("({lhs}) * ({rhs})")
            }
            Operation::Exp(l_idx, r_idx) => {
                let lhs = l_idx.to_string(ir, elem_type);
                match elem_type {
                    ElemType::Base => format!("({lhs}).exp(Felt::new({r_idx}))"),
                    ElemType::Ext => format!("({lhs}).exp(E::PositiveInteger::from({r_idx}_u64))"),
                }
            }
        }
    }
}
