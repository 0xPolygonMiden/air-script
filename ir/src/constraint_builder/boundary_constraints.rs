use super::{
    ast::BoundaryStmt, ConstraintBuilder, ConstraintBuilderContext, ConstraintDomain, Expression,
    SemanticError, TraceAccess, TraceSegment,
};
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
                let (boundary, trace_access, expression) = constraint.into_parts();
                let domain = boundary.into();
                self.context = ConstraintBuilderContext::BoundaryConstraint(domain);

                let trace_access = self.symbol_table.get_trace_binding_access(&trace_access)?;
                self.add_constrained_boundary(trace_access, domain)?;

                self.insert_constraint(Expression::TraceAccess(trace_access), expression)?;
            }
            BoundaryStmt::Variable(variable) => self.symbol_table.insert_variable(variable)?,
            BoundaryStmt::ConstraintComprehension(_, _) => todo!(),
        }

        Ok(())
    }

    /// TODO: docs
    fn add_constrained_boundary(
        &mut self,
        trace_access: TraceAccess,
        domain: ConstraintDomain,
    ) -> Result<(), SemanticError> {
        let constrained_boundary =
            ConstrainedBoundary::new(trace_access.trace_segment(), trace_access.col_idx(), domain);
        // add the boundary to the set of constrained boundaries.
        if !self.constrained_boundaries.insert(constrained_boundary) {
            // raise an error if the same boundary was previously constrained
            return Err(SemanticError::boundary_already_constrained(
                &constrained_boundary,
            ));
        }

        Ok(())
    }
}
