use super::{ast::BoundaryStmt, ConstraintBuilder, ConstraintDomain, SemanticError, TraceSegment};
use std::fmt::Display;

/// [ConstrainedBoundary] represents the location within the trace where a boundary constraint is
/// applied. It identifies the trace segment, the trace column index, and the [ConstraintDomain].
/// The [ConstraintDomain] is assumed to be a valid boundary, either FirstRow or LastRow.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct ConstrainedBoundary {
    trace_segment: TraceSegment,
    col_idx: usize,
    domain: ConstraintDomain,
}

impl ConstrainedBoundary {
    pub fn new(trace_segment: TraceSegment, col_idx: usize, domain: ConstraintDomain) -> Self {
        debug_assert!(domain.is_boundary());
        Self {
            trace_segment,
            col_idx,
            domain,
        }
    }
}

impl Display for ConstrainedBoundary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} of column {} in segment {}",
            self.domain, self.col_idx, self.trace_segment
        )
    }
}

impl ConstraintBuilder {
    /// Adds the provided parsed boundary statement to the graph. The statement can either be a
    /// variable defined in the boundary constraints section or a boundary constraint expression.
    ///
    /// In case the statement is a variable, it is added to the symbol table.
    ///
    /// In case the statement is a constraint, the constraint is turned into a subgraph which is
    /// added to the [AlgebraicGraph] (reusing any existing nodes). The index of its entry node
    /// is then saved in the boundary_constraints matrix.
    pub(super) fn process_boundary_stmt(
        &mut self,
        stmt: BoundaryStmt,
    ) -> Result<(), SemanticError> {
        match stmt {
            BoundaryStmt::Constraint(constraint) => {
                let (boundary, access, value, _) = constraint.into_parts();

                let trace_access = self.symbol_table.get_trace_access(&access)?;
                let domain = boundary.into();
                let constrained_boundary = ConstrainedBoundary::new(
                    trace_access.trace_segment(),
                    trace_access.col_idx(),
                    domain,
                );
                // add the boundary to the set of constrained boundaries.
                if !self.constrained_boundaries.insert(constrained_boundary) {
                    // raise an error if the same boundary was previously constrained
                    return Err(SemanticError::boundary_already_constrained(
                        &constrained_boundary,
                    ));
                }

                // add the trace access at the specified boundary to the graph.
                let lhs = self.insert_trace_access(&trace_access)?;

                // get the trace segment and domain of the boundary column access
                let (lhs_segment, lhs_domain) = self.graph.node_details(&lhs, domain)?;
                debug_assert!(
                   lhs_domain == domain,
                   "The boundary constraint's domain should be {lhs_domain:?}, but the domain {domain:?} was inferred by the graph",
               );

                // add its expression to the constraints graph.
                let rhs = self.insert_expr(value)?;
                // get the trace segment and domain of the expression
                let (rhs_segment, rhs_domain) = self.graph.node_details(&rhs, domain)?;

                // ensure that the inferred trace segment and domain of the rhs expression can be
                // applied to column against which the boundary constraint is applied.
                if lhs_segment < rhs_segment {
                    // trace segment inference defaults to the lowest segment (the main trace) and is
                    // adjusted according to the use of random values and trace columns.
                    return Err(SemanticError::trace_segment_mismatch(lhs_segment));
                }
                if lhs_domain != rhs_domain {
                    return Err(SemanticError::incompatible_constraint_domains(
                        &lhs_domain,
                        &rhs_domain,
                    ));
                }

                // merge the two sides of the expression into a constraint.
                let root = self.merge_equal_exprs(lhs, rhs);

                // save the constraint information
                self.insert_constraint(root, lhs_segment.into(), domain)?
            }
            BoundaryStmt::VariableBinding(variable) => {
                self.symbol_table.insert_variable(variable)?
            }
        }

        Ok(())
    }
}
