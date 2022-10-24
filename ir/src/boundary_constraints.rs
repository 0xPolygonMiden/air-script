use std::collections::BTreeMap;

use super::{BoundaryExpr, Identifier, PublicInputs, SemanticError, TraceColumns};
use parser::ast;

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
    /// Returns an error if:
    /// - The column specified for the boundary constraint has not been declared.
    /// - The constraint expression is contains invalid public input references.
    /// - A boundary constraint has already been declared for the specified column and boundary.
    pub(super) fn insert(
        &mut self,
        constraint: &ast::BoundaryConstraint,
        trace_columns: &TraceColumns,
        pub_inputs: &PublicInputs,
    ) -> Result<(), SemanticError> {
        let (trace_col_idx, expr) = validate_constraint(constraint, trace_columns, pub_inputs)?;

        // add the constraint to the specified boundary
        match constraint.boundary() {
            ast::Boundary::First => {
                if self.first.insert(trace_col_idx, expr).is_some() {
                    return Err(SemanticError::RedefinedBoundary(format!(
                        "Boundary constraint redefined for {} at first step",
                        constraint.column()
                    )));
                }
            }
            ast::Boundary::Last => {
                if self.last.insert(trace_col_idx, expr).is_some() {
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

/// Validates the specified constraint against the provided set of trace columns and public inputs.
///
/// # Errors
/// Returns an error if the constraint is specified against a column that hasn't been declared or if
/// the constraint's expression is invalid.
fn validate_constraint(
    constraint: &ast::BoundaryConstraint,
    trace_columns: &TraceColumns,
    pub_inputs: &PublicInputs,
) -> Result<(usize, BoundaryExpr), SemanticError> {
    let col_idx = trace_columns.get_column_index(constraint.column())?;
    let value = constraint.value();
    validate_expression(&value, pub_inputs)?;

    Ok((col_idx, value))
}

/// Recursively validates the BoundaryExpression.
///
/// # Errors
/// Returns an error if the expression includes a reference to a public input that hasn't been
/// declared or to an invalid index in an existing public input.
fn validate_expression(
    expr: &ast::BoundaryExpr,
    pub_inputs: &PublicInputs,
) -> Result<(), SemanticError> {
    match expr {
        BoundaryExpr::PublicInput(Identifier(name), index) => {
            pub_inputs.validate_input(name, *index)?;
        }
        BoundaryExpr::Add(lhs, rhs) | BoundaryExpr::Subtract(lhs, rhs) => {
            validate_expression(lhs, pub_inputs)?;
            validate_expression(rhs, pub_inputs)?;
        }
        _ => {}
    };

    Ok(())
}
