use air_ir::{
    Air, ConstraintDomain, ConstraintRoot, NodeIndex, Operation, TraceAccess, TraceSegmentId, Value,
};

use crate::air::ElemType;

// RUST STRING GENERATION FOR THE CONSTRAINT GRAPH
// ================================================================================================

/// Code generation trait for generating Rust code strings from IR types related to constraints and
/// the [AlgebraicGraph].
pub trait Codegen {
    fn to_string(&self, ir: &Air, elem_type: ElemType) -> String;
}

impl Codegen for TraceAccess {
    fn to_string(&self, _ir: &Air, elem_type: ElemType) -> String {
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
        match elem_type {
            ElemType::Base => format!("{frame}_{row_offset}.clone().into()"),
            ElemType::Ext => format!("AB::ExprEF::from({frame}_{row_offset}.clone().into())"),
        }
    }
}

impl Codegen for NodeIndex {
    fn to_string(&self, ir: &Air, elem_type: ElemType) -> String {
        let op = ir.constraint_graph().node(self).op();
        op.to_string(ir, elem_type)
    }
}

impl Codegen for Operation {
    fn to_string(&self, ir: &Air, elem_type: ElemType) -> String {
        match self {
            Operation::Value(value) => value.to_string(ir, elem_type),
            Operation::Add(..) => binary_op_to_string(ir, elem_type, self),
            Operation::Sub(..) => binary_op_to_string(ir, elem_type, self),
            Operation::Mul(..) => binary_op_to_string(ir, elem_type, self),
        }
    }
}

impl Codegen for Value {
    fn to_string(&self, ir: &Air, elem_type: ElemType) -> String {
        match self {
            Value::Constant(0) => match elem_type {
                ElemType::Base => format!("AB::Expr::ZERO"),
                ElemType::Ext => format!("AB::ExprEF::ZERO"),
            },
            Value::Constant(1) => match elem_type {
                ElemType::Base => format!("AB::Expr::ONE"),
                ElemType::Ext => format!("AB::ExprEF::ONE"),
            },
            Value::Constant(value) => match elem_type {
                ElemType::Base => format!("AB::Expr::from_u64({value})"),
                ElemType::Ext => format!("AB::ExprEF::from_u64({value})"),
            },
            Value::TraceAccess(trace_access) => trace_access.to_string(ir, elem_type),
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
                format!("periodic_values[{index}].clone().into()")
            },
            Value::PublicInputTable(public_input_table_access) => {
                let idx = ir
                    .reduced_public_input_table_accesses()
                    .iter()
                    .position(|pi| pi == public_input_table_access)
                    .unwrap();
                format!("aux_bus_boundary_values[{idx}].into()")
            },
            Value::RandomValue(idx) => {
                if *idx == 0 {
                    format!("alpha.into()")
                } else {
                    format!("beta_challenges[{}].into()", idx - 1)
                }
            },
        }
    }
}

/// Returns a string representation of a binary operation.
fn binary_op_to_string(ir: &Air, elem_type: ElemType, op: &Operation) -> String {
    match op {
        Operation::Add(l_idx, r_idx) => {
            let lhs = l_idx.to_string(ir, elem_type);
            let rhs = r_idx.to_string(ir, elem_type);
            format!("{lhs} + {rhs}")
        },
        Operation::Sub(l_idx, r_idx) => {
            let lhs = l_idx.to_string(ir, elem_type);
            let rhs = if ir.constraint_graph().node(r_idx).op().precedence() <= op.precedence() {
                format!("({})", r_idx.to_string(ir, elem_type))
            } else {
                r_idx.to_string(ir, elem_type)
            };
            format!("{lhs} - {rhs}")
        },
        Operation::Mul(l_idx, r_idx) => {
            let lhs_op = ir.constraint_graph().node(l_idx).op();
            let rhs_op = ir.constraint_graph().node(r_idx).op();

            let lhs = if lhs_op.precedence() < op.precedence() {
                format!("({})", l_idx.to_string(ir, elem_type))
            } else {
                l_idx.to_string(ir, elem_type)
            };
            let rhs = if rhs_op.precedence() < op.precedence() {
                format!("({})", r_idx.to_string(ir, elem_type))
            } else {
                r_idx.to_string(ir, elem_type)
            };

            match (lhs_op, rhs_op) {
                (_, Operation::Value(Value::Constant(2))) => format!("{lhs}.double()"),
                (Operation::Value(Value::Constant(2)), _) => format!("{rhs}.double()"),
                _ => format!("{lhs} * {rhs}"),
            }
        },
        _ => panic!("unsupported operation"),
    }
}

/// Recursively determines if the expression depends on extension field values (i.e., aux trace,
/// random values, periodic columns, or public input tables).
pub fn needs_extension_field(ir: &Air, expr_root: NodeIndex) -> bool {
    let op = ir.constraint_graph().node(&expr_root).op();
    match op {
        Operation::Value(value) => match value {
            Value::TraceAccess(trace_access) => trace_access.segment == TraceSegmentId::Aux,
            Value::Constant(_) => false,
            Value::PeriodicColumn(_) => false,
            Value::PublicInput(_) => false,
            Value::PublicInputTable(_) => true,
            Value::RandomValue(_) => true,
        },
        Operation::Add(lhs, rhs) | Operation::Sub(lhs, rhs) | Operation::Mul(lhs, rhs) => {
            needs_extension_field(ir, *lhs) || needs_extension_field(ir, *rhs)
        },
    }
}

/// Returns the appropriate domain flag string for the given [ConstraintDomain].
fn get_boundary_domain_flag_str(domain: &ConstraintDomain) -> &'static str {
    match domain {
        ConstraintDomain::FirstRow => ".when_first_row()",
        ConstraintDomain::LastRow => ".when_last_row()",
        _ => unreachable!("Invalid domain for boundary constraints"),
    }
}

/// Returns the appropriate domain flag string for the given [ConstraintDomain].
fn get_integrity_domain_flag_str(domain: &ConstraintDomain) -> &'static str {
    match domain {
        ConstraintDomain::EveryFrame(_) => ".when_transition()",
        ConstraintDomain::EveryRow => "",
        _ => unreachable!("Invalid domain for integrity constraints"),
    }
}

pub fn constraint_to_string(ir: &Air, constraint: &ConstraintRoot, in_boundary: bool) -> String {
    let expr_root = constraint.node_index();
    let needs_extension_field = needs_extension_field(ir, *expr_root);
    let elem_type = if needs_extension_field {
        ElemType::Ext
    } else {
        ElemType::Base
    };
    let expr_root_string = expr_root.to_string(ir, elem_type);

    // If the constraint is a transition constraint (depends on the next row), we do not
    // evaluate it in the last row, with the `when_transition` method.
    let domain_flag = if in_boundary {
        get_boundary_domain_flag_str(&constraint.domain())
    } else {
        get_integrity_domain_flag_str(&constraint.domain())
    };
    let extension_field_flag = if needs_extension_field { "_ext" } else { "" };
    let assertion =
        format!("builder{domain_flag}.assert_zero{extension_field_flag}({expr_root_string});");
    assertion
}
