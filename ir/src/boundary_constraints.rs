use super::{BoundaryExpr, SemanticError, TraceColumns};
use parser::ast;
use std::collections::BTreeMap;

// BOUNDARY CONSTRAINTS
// ================================================================================================

/// A struct containing all of the boundary constraints to be applied at each of the 2 allowed
/// boundaries (first row and last row). For ease of code generation and evaluation, constraints are
/// sorted into maps by the boundary. This also simplifies ensuring that there are no conflicting
/// constraints sharing a boundary and column index.
#[derive(Default, Debug)]
pub(crate) struct BoundaryConstraints {
    /// The boundary constraints to be applied at the first row of the trace, with the trace column
    /// index as the key, and the expression as the value.
    first: BTreeMap<usize, BoundaryExpr>,
    /// The boundary constraints to be applied at the last row of the trace, with the trace column
    /// index as the key, and the expression as the value.
    last: BTreeMap<usize, BoundaryExpr>,
}

impl BoundaryConstraints {
    // --- ACCESSORS ------------------------------------------------------------------------------

    /// Returns the total number of boundary constraints
    pub fn len(&self) -> usize {
        self.first.len() + self.last.len()
    }

    /// Returns all of the boundary constraints for the first row of the trace.
    pub fn first(&self) -> Vec<&BoundaryExpr> {
        self.first.values().collect()
    }

    /// Returns all of the boundary constraints for the final row of the trace.
    pub fn last(&self) -> Vec<&BoundaryExpr> {
        self.last.values().collect()
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// Add a boundary constraint from the AST to the list of constraints for its specified
    /// boundary.
    ///
    /// # Errors
    /// Returns an error if the boundary constraint references an identifier, which is not allowed.
    pub(super) fn insert(
        &mut self,
        constraint: &ast::BoundaryConstraint,
        trace_columns: &TraceColumns,
    ) -> Result<(), SemanticError> {
        let col_idx = trace_columns.get_column_index(constraint.column())?;
        let value = constraint.value();

        // add the constraint to the specified boundary
        match constraint.boundary() {
            ast::Boundary::First => {
                if self.first.insert(col_idx, value).is_some() {
                    return Err(SemanticError::RedefinedBoundary(format!(
                        "Boundary constraint redefined for {} at first step",
                        constraint.column()
                    )));
                }
            }
            ast::Boundary::Last => {
                if self.last.insert(col_idx, value).is_some() {
                    return Err(SemanticError::RedefinedBoundary(format!(
                        "Boundary constraint redefined for {} at last step",
                        constraint.column()
                    )));
                }
            }
        }

        Ok(())
    }
}
