use super::{BTreeMap, BoundaryExpr, Identifier, IdentifierType, SemanticError, SymbolTable};
use parser::ast;

// BOUNDARY CONSTRAINTS
// ================================================================================================

/// A struct containing all of the boundary constraints to be applied at each of the 2 allowed
/// boundaries (first row and last row). For ease of code generation and evaluation, constraints are
/// sorted into maps by the boundary. This also simplifies ensuring that there are no conflicting
/// constraints sharing a boundary and column index.
#[derive(Default, Debug)]
pub(crate) struct BoundaryConstraints {
    /// The boundary constraints to be applied at the first row of the main trace, with the trace
    /// column index as the key, and the expression as the value.
    main_first: BTreeMap<usize, BoundaryExpr>,
    /// The boundary constraints to be applied at the last row of the main trace, with the trace
    /// column index as the key, and the expression as the value.
    main_last: BTreeMap<usize, BoundaryExpr>,
    /// The boundary constraints to be applied at the first row of the aux trace, with the trace
    /// column index as the key, and the expression as the value.
    aux_first: BTreeMap<usize, BoundaryExpr>,
    /// The boundary constraints to be applied at the last row of the aux trace, with the trace
    /// column index as the key, and the expression as the value.
    aux_last: BTreeMap<usize, BoundaryExpr>,
}

impl BoundaryConstraints {
    // --- ACCESSORS ------------------------------------------------------------------------------

    /// Returns the total number of boundary constraints for the main trace.
    pub fn main_len(&self) -> usize {
        self.main_first.len() + self.main_last.len()
    }

    /// Returns all of the boundary constraints for the first row of the main trace.
    pub fn main_first(&self) -> Vec<(usize, &BoundaryExpr)> {
        self.main_first.iter().map(|(k, v)| (*k, v)).collect()
    }

    /// Returns all of the boundary constraints for the final row of the main trace.
    pub fn main_last(&self) -> Vec<(usize, &BoundaryExpr)> {
        self.main_last.iter().map(|(k, v)| (*k, v)).collect()
    }

    /// Returns the total number of boundary constraints for the aux trace.
    pub fn aux_len(&self) -> usize {
        self.aux_first.len() + self.aux_last.len()
    }

    /// Returns all of the boundary constraints for the first row of the aux trace.
    pub fn aux_first(&self) -> Vec<(usize, &BoundaryExpr)> {
        self.aux_first.iter().map(|(k, v)| (*k, v)).collect()
    }

    /// Returns all of the boundary constraints for the final row of the aux trace.
    pub fn aux_last(&self) -> Vec<(usize, &BoundaryExpr)> {
        self.aux_last.iter().map(|(k, v)| (*k, v)).collect()
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// Add a boundary constraint from the AST to the list of constraints for its specified
    /// boundary.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The identifier specified for the boundary constraint column has not been declared or has
    ///   been declared with the wrong type.
    /// - The constraint expression is contains invalid public input references.
    /// - A boundary constraint has already been declared for the specified column and boundary.
    pub(super) fn insert(
        &mut self,
        symbol_table: &SymbolTable,
        constraint: &ast::BoundaryConstraint,
    ) -> Result<(), SemanticError> {
        // validate the expression
        let expr = constraint.value();
        validate_expression(symbol_table, &expr)?;

        // add the constraint to the specified boundary for the specified trace
        let col_type = symbol_table.get_type(constraint.column())?;
        let result = match col_type {
            IdentifierType::MainTraceColumn(col_idx) => match constraint.boundary() {
                ast::Boundary::First => self.main_first.insert(col_idx, expr),
                ast::Boundary::Last => self.main_last.insert(col_idx, expr),
            },
            IdentifierType::AuxTraceColumn(col_idx) => match constraint.boundary() {
                ast::Boundary::First => self.aux_first.insert(col_idx, expr),
                ast::Boundary::Last => self.aux_last.insert(col_idx, expr),
            },
            _ => {
                return Err(SemanticError::InvalidUsage(format!(
                    "Identifier {} was declared as a {}, not as a trace column",
                    constraint.column(),
                    col_type
                )));
            }
        };

        // raise an error if multiple constraints were applied to the same boundary
        if result.is_some() {
            return Err(SemanticError::TooManyConstraints(format!(
                "A boundary constraint was already defined for {} '{}' at the {}",
                col_type,
                constraint.column(),
                constraint.boundary()
            )));
        }

        Ok(())
    }
}

/// Recursively validates the BoundaryExpression.
///
/// # Errors
/// Returns an error if the expression includes a reference to a public input that hasn't been
/// declared or to an invalid index in an existing public input.
fn validate_expression(
    symbol_table: &SymbolTable,
    expr: &ast::BoundaryExpr,
) -> Result<(), SemanticError> {
    match expr {
        BoundaryExpr::PubInput(Identifier(name), index) => {
            symbol_table.validate_public_input(name, *index)
        }
        BoundaryExpr::Add(lhs, rhs) | BoundaryExpr::Sub(lhs, rhs) => {
            validate_expression(symbol_table, lhs)?;
            validate_expression(symbol_table, rhs)
        }
        _ => Ok(()),
    }
}
