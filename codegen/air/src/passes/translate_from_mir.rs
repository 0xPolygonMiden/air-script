use air_parser::{ast::{self, TraceSegment}, LexicalScope};
use air_pass::Pass;

use miden_diagnostics::{DiagnosticsHandler, Severity, SourceSpan, Span, Spanned};
use mir::{Mir, SpannedMirValue};

use crate::{graph::NodeIndex, ir::*, CompileError};

pub struct MirToAir<'a> {
    diagnostics: &'a DiagnosticsHandler,
}
impl<'a> MirToAir<'a> {
    /// Create a new instance of this pass
    #[inline]
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}
impl<'p> Pass for MirToAir<'p> {
    type Input<'a> = Mir;
    type Output<'a> = Air;
    type Error = CompileError;

    fn run<'a>(&mut self, mir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut air = Air::new(mir.name);
        //TODO: Implement MIR > AIR lowering

        air.trace_segment_widths = mir.trace_columns.iter().map(|ts| ts.size as u16).collect();
        air.num_random_values =  mir.num_random_values;
        air.periodic_columns = mir.periodic_columns.clone();
        air.public_inputs = mir.public_inputs.clone();

        let mut builder = AirBuilder {
            diagnostics: self.diagnostics,
            air: &mut air,
            mir: &mir,
            trace_columns: mir.trace_columns.clone(),
        };

        let graph = mir.constraint_graph();

        for bc in graph.boundary_constraints_roots.iter() {
            builder.build_boundary_constraint(bc)?;
        }

        for ic in graph.integrity_constraints_roots.iter() {
            builder.build_integrity_constraint(ic)?;
        }

        Ok(air)
    }
}

#[derive(Debug, Clone)]
enum MemoizedBinding {
    /// The binding was reduced to a node in the graph
    Scalar(NodeIndex),
    /// The binding represents a vector of nodes in the graph
    Vector(Vec<NodeIndex>),
    /// The binding represents a matrix of nodes in the graph
    Matrix(Vec<Vec<NodeIndex>>),
}

struct AirBuilder<'a> {
    diagnostics: &'a DiagnosticsHandler,
    air: &'a mut Air,
    mir: &'a Mir,
    trace_columns: Vec<TraceSegment>,
}

impl<'a> AirBuilder<'a> {

    fn insert_mir_operation(&mut self, mir_node: &mir::NodeIndex) -> NodeIndex {
        let mir_op = self.mir.constraint_graph().node(mir_node).op();
        match mir_op {
            mir::Operation::Value(spanned_mir_value) => {
                let mir_value = &spanned_mir_value.value;

                let value = match mir_value {
                    mir::MirValue::Constant(constant_value) => {
                        if let mir::ConstantValue::Felt(felt) = constant_value {
                            Value::Constant(*felt)
                        } else {
                            unreachable!()
                        }
                    },
                    mir::MirValue::TraceAccess(trace_access) => {
                        Value::TraceAccess(
                            TraceAccess { 
                                segment: trace_access.segment,
                                column: trace_access.column,
                                row_offset: trace_access.row_offset
                            }
                        )
                    },
                    mir::MirValue::PeriodicColumn(periodic_column_access) => {
                        Value::PeriodicColumn(
                            PeriodicColumnAccess {
                                name: periodic_column_access.name.clone(),
                                cycle: periodic_column_access.cycle
                            }
                        )
                    },
                    mir::MirValue::PublicInput(public_input_access) => {
                        Value::PublicInput(
                            PublicInputAccess {
                                name: public_input_access.name.clone(),
                                index: public_input_access.index
                            }
                        )
                    },
                    mir::MirValue::RandomValue(rv) => {
                        Value::RandomValue(*rv)
                    },
                    
                    /*mir::MirValue::TraceAccessBinding(trace_access_binding) => {
                        if trace_access_binding.size == 1 {
                            Value::TraceAccess(
                                TraceAccess {
                                    segment: trace_access_binding.segment,
                                    column: trace_access_binding.offset,
                                    row_offset: 0,
                                }
                            )
                        } else {
                            unreachable!();
                        }
                    },*/
                    _ => unreachable!(),
                    /*mir::MirValue::TraceAccessBinding(trace_access_binding) => todo!(),
                    mir::MirValue::RandomValueBinding(random_value_binding) => todo!(),
                    mir::MirValue::Vector(vec) => todo!(),
                    mir::MirValue::Matrix(vec) => todo!(),
                    mir::MirValue::Variable(mir_type, _, node_index) => todo!(),
                    mir::MirValue::Definition(vec, node_index, node_index1) => todo!(),*/

                };

                return self.insert_op(Operation::Value(value));
            },
            mir::Operation::Add(lhs, rhs) => {
                let lhs_node_index = self.insert_mir_operation(lhs);
                let rhs_node_index = self.insert_mir_operation(rhs);
                return self.insert_op(Operation::Add(lhs_node_index, rhs_node_index));
            },
            mir::Operation::Sub(lhs, rhs) => {
                let lhs_node_index = self.insert_mir_operation(lhs);
                let rhs_node_index = self.insert_mir_operation(rhs);
                return self.insert_op(Operation::Sub(lhs_node_index, rhs_node_index));
            },
            mir::Operation::Mul(lhs, rhs) => {
                let lhs_node_index = self.insert_mir_operation(lhs);
                let rhs_node_index = self.insert_mir_operation(rhs);
                return self.insert_op(Operation::Mul(lhs_node_index, rhs_node_index));
            },
            _ => unreachable!(),
        }
    }

    fn build_boundary_constraint(&mut self, bc: &mir::NodeIndex) -> Result<(), CompileError> {

        let bc_op = self.mir.constraint_graph().node(bc).op();

        match bc_op {
            mir::Operation::Vector(vec) => {
                for node in vec.iter() {
                    self.build_boundary_constraint(node)?;
                }
                return Ok(());
            },
            mir::Operation::Matrix(m) => {
                for row in m.iter() {
                    for node in row.iter() {
                        self.build_boundary_constraint(node)?;
                    }
                }
                return Ok(());
            },
            mir::Operation::Enf(child_node_index) => {
                let mir_op = self.mir.constraint_graph().node(child_node_index).op();

                let mir::Operation::Sub(lhs, rhs) = mir_op else {
                    unreachable!(); // Raise diag
                };

                // Check that lhs is a Bounded trace access
                // TODO: Put in a helper function
                let lhs_op = self.mir.constraint_graph().node(lhs).op();
                let mir::Operation::Boundary(boundary, trace_access_index) = lhs_op else {
                    unreachable!(); // Raise diag
                };
                let trace_access_op = self.mir.constraint_graph().node(trace_access_index).op();
                let mir::Operation::Value(
                    SpannedMirValue {
                        value: mir::MirValue::TraceAccess(trace_access),
                        span: lhs_span,
                    }
                ) = trace_access_op else {
                    unreachable!(); // Raise diag
                };

                if let Some(prev) = self.trace_columns[trace_access.segment].mark_constrained(
                    *lhs_span,
                    trace_access.column,
                    *boundary,
                ) {
                    self.diagnostics
                        .diagnostic(Severity::Error)
                        .with_message("overlapping boundary constraints")
                        .with_primary_label(
                            *lhs_span,
                            "this constrains a column and boundary that has already been constrained",
                        )
                        .with_secondary_label(prev, "previous constraint occurs here")
                        .emit();
                    return Err(CompileError::Failed);
                }

                let lhs = self.air.constraint_graph_mut().insert_node(Operation::Value(Value::TraceAccess(
                    TraceAccess {
                        segment: trace_access.segment,
                        column: trace_access.column,
                        row_offset: trace_access.row_offset,
                    }
                )));
                let rhs = self.insert_mir_operation(rhs);

                // Compare the inferred trace segment and domain of the operands                
                let domain = (*boundary).into();
                {
                    let graph = self.air.constraint_graph();
                    let (lhs_segment, lhs_domain) = graph.node_details(&lhs, domain)?;
                    let (rhs_segment, rhs_domain) = graph.node_details(&rhs, domain)?;
                    if lhs_segment < rhs_segment {
                        // trace segment inference defaults to the lowest segment (the main trace) and is
                        // adjusted according to the use of random values and trace columns.
                        let lhs_segment_name = self.trace_columns[lhs_segment].name;
                        let rhs_segment_name = self.trace_columns[rhs_segment].name;
                        self.diagnostics.diagnostic(Severity::Error)
                            .with_message("invalid boundary constraint")
                            .with_primary_label(*lhs_span, format!("this constrains a column in the '{lhs_segment_name}' trace segment"))
                            .with_secondary_label(SourceSpan::UNKNOWN, format!("but this expression implies the '{rhs_segment_name}' trace segment"))
                            .with_note("Boundary constraints require both sides of the constraint to apply to the same trace segment.")
                            .emit();
                        return Err(CompileError::Failed);
                    }
                    if lhs_domain != rhs_domain {
                        self.diagnostics.diagnostic(Severity::Error)
                            .with_message("invalid boundary constraint")
                            .with_primary_label(*lhs_span, format!("this has a constraint domain of {lhs_domain}"))
                            .with_secondary_label(SourceSpan::UNKNOWN, format!("this has a constraint domain of {rhs_domain}"))
                            .with_note("Boundary constraints require both sides of the constraint to be in the same domain.")
                            .emit();
                        return Err(CompileError::Failed);
                    }
                }

                // Merge the expressions into a single constraint
                let root = self.insert_op(Operation::Sub(lhs, rhs));

                // Store the generated constraint
                self.air
                    .constraints
                    .insert_constraint(trace_access.segment, root, domain);
            },
            mir::Operation::Sub(lhs, rhs) => {

                // Check that lhs is a Bounded trace access
                // TODO: Put in a helper function
                let lhs_op = self.mir.constraint_graph().node(lhs).op();
                let mir::Operation::Boundary(boundary, trace_access_index) = lhs_op else {
                    unreachable!(); // Raise diag
                };
                let trace_access_op = self.mir.constraint_graph().node(trace_access_index).op();

                let (trace_access, lhs_span) = match trace_access_op {
                    mir::Operation::Value(
                        SpannedMirValue {
                            value: mir::MirValue::TraceAccess(trace_access),
                            span: lhs_span,
                        }
                    ) => (*trace_access, lhs_span),
                    
                    mir::Operation::Value(
                        SpannedMirValue {
                            value: mir::MirValue::TraceAccessBinding(trace_access_binding),
                            span: lhs_span,
                        }
                    ) => {
                        if trace_access_binding.size != 1 {
                            self.diagnostics.diagnostic(Severity::Error)
                                .with_message("invalid boundary constraint")
                                .with_primary_label(*lhs_span, "this has a trace access binding with a size greater than 1")
                                .with_note("Boundary constraints require both sides of the constraint to be single columns.")
                                .emit();
                            return Err(CompileError::Failed);
                        }
                        let trace_access = mir::TraceAccess {
                            segment: trace_access_binding.segment,
                            column: trace_access_binding.offset,
                            row_offset: 0,
                        };
                        (trace_access, lhs_span)
                    },                    
                    _ => unreachable!("Expected TraceAccess, received {:?}", trace_access_op), // Raise diag
                };

                if let Some(prev) = self.trace_columns[trace_access.segment].mark_constrained(
                    *lhs_span,
                    trace_access.column,
                    *boundary,
                ) {
                    self.diagnostics
                        .diagnostic(Severity::Error)
                        .with_message("overlapping boundary constraints")
                        .with_primary_label(
                            *lhs_span,
                            "this constrains a column and boundary that has already been constrained",
                        )
                        .with_secondary_label(prev, "previous constraint occurs here")
                        .emit();
                    return Err(CompileError::Failed);
                }

                let lhs = self.air.constraint_graph_mut().insert_node(Operation::Value(Value::TraceAccess(
                    TraceAccess {
                        segment: trace_access.segment,
                        column: trace_access.column,
                        row_offset: trace_access.row_offset,
                    }
                )));
                let rhs = self.insert_mir_operation(rhs);

                // Compare the inferred trace segment and domain of the operands                
                let domain = (*boundary).into();
                {
                    let graph = self.air.constraint_graph();
                    let (lhs_segment, lhs_domain) = graph.node_details(&lhs, domain)?;
                    let (rhs_segment, rhs_domain) = graph.node_details(&rhs, domain)?;
                    if lhs_segment < rhs_segment {
                        // trace segment inference defaults to the lowest segment (the main trace) and is
                        // adjusted according to the use of random values and trace columns.
                        let lhs_segment_name = self.trace_columns[lhs_segment].name;
                        let rhs_segment_name = self.trace_columns[rhs_segment].name;
                        self.diagnostics.diagnostic(Severity::Error)
                            .with_message("invalid boundary constraint")
                            .with_primary_label(*lhs_span, format!("this constrains a column in the '{lhs_segment_name}' trace segment"))
                            .with_secondary_label(SourceSpan::UNKNOWN, format!("but this expression implies the '{rhs_segment_name}' trace segment"))
                            .with_note("Boundary constraints require both sides of the constraint to apply to the same trace segment.")
                            .emit();
                        return Err(CompileError::Failed);
                    }
                    if lhs_domain != rhs_domain {
                        self.diagnostics.diagnostic(Severity::Error)
                            .with_message("invalid boundary constraint")
                            .with_primary_label(*lhs_span, format!("this has a constraint domain of {lhs_domain}"))
                            .with_secondary_label(SourceSpan::UNKNOWN, format!("this has a constraint domain of {rhs_domain}"))
                            .with_note("Boundary constraints require both sides of the constraint to be in the same domain.")
                            .emit();
                        return Err(CompileError::Failed);
                    }
                }

                // Merge the expressions into a single constraint
                let root = self.insert_op(Operation::Sub(lhs, rhs));

                // Store the generated constraint
                self.air
                    .constraints
                    .insert_constraint(trace_access.segment, root, domain);
            },
            _ => unreachable!("{:?}", bc_op),
        }

        Ok(())
    }

    fn build_integrity_constraint(&mut self, ic: &mir::NodeIndex) -> Result<(), CompileError> {
        let ic_op = self.mir.constraint_graph().node(ic).op();

        match ic_op {
            mir::Operation::Vector(vec) => {
                for node in vec.iter() {
                    self.build_integrity_constraint(node)?;
                }
                return Ok(());
            },
            mir::Operation::Matrix(m) => {
                for row in m.iter() {
                    for node in row.iter() {
                        self.build_integrity_constraint(node)?;
                    }
                }
                return Ok(());
            },
            mir::Operation::Enf(child_node_index) => {
                let mir_op = self.mir.constraint_graph().node(child_node_index).op();

                match mir_op {
                    mir::Operation::Sub(lhs, rhs) => {
                        let lhs_node_index = self.insert_mir_operation(lhs);
                        let rhs_node_index = self.insert_mir_operation(rhs);
                        let root = self.insert_op(Operation::Sub(lhs_node_index, rhs_node_index));
                        let (trace_segment, domain) = self
                            .air
                            .constraint_graph()
                            .node_details(&root, ConstraintDomain::EveryRow)?;
                        self.air
                            .constraints
                            .insert_constraint(trace_segment, root, domain);
                    },
                    mir::Operation::If(cond, then, else_) => {
                        let cond_node_index = self.insert_mir_operation(cond);
                        let then_node_index = self.insert_mir_operation(then);
                        let else_node_index = self.insert_mir_operation(else_);

                        let pos_root = self.insert_op(Operation::Mul(then_node_index, cond_node_index));
                        let one = self.insert_op(Operation::Value(Value::Constant(1)));
                        let neg_cond = self.insert_op(Operation::Sub(one, cond_node_index));
                        let neg_root = self.insert_op(Operation::Mul(else_node_index, neg_cond));

                        let (trace_segment, domain) = self
                            .air
                            .constraint_graph()
                            .node_details(&pos_root, ConstraintDomain::EveryRow)?;
                        self.air
                            .constraints
                            .insert_constraint(trace_segment, pos_root, domain);
                        let (trace_segment, domain) = self
                            .air
                            .constraint_graph()
                            .node_details(&neg_root, ConstraintDomain::EveryRow)?;
                        self.air
                            .constraints
                            .insert_constraint(trace_segment, neg_root, domain);
                    }
                    _ => unreachable!()
                }
            },
            
            mir::Operation::Sub(lhs, rhs) => {
                        let lhs_node_index = self.insert_mir_operation(lhs);
                        let rhs_node_index = self.insert_mir_operation(rhs);
                        let root = self.insert_op(Operation::Sub(lhs_node_index, rhs_node_index));
                        let (trace_segment, domain) = self
                            .air
                            .constraint_graph()
                            .node_details(&root, ConstraintDomain::EveryRow)?;
                        self.air
                            .constraints
                            .insert_constraint(trace_segment, root, domain);
            },
            _ => todo!()
        }

        Ok(())
    }

    /// Adds the specified operation to the graph and returns the index of its node.
    #[inline]
    fn insert_op(&mut self, op: Operation) -> NodeIndex {
        self.air.constraint_graph_mut().insert_node(op)
    }
}
