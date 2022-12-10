use crate::TraceSegment;

use super::{BTreeMap, BoundaryExpr, IdentifierType, SemanticError, SymbolTable};
use parser::ast::{self, BoundaryStmt, BoundaryVariable, BoundaryVariableType};

// BOUNDARY CONSTRAINTS
// ================================================================================================

/// A struct containing all of the boundary constraints to be applied at each of the 2 allowed
/// boundaries (first row and last row). For ease of code generation and evaluation, constraints are
/// sorted into maps by the boundary. This also simplifies ensuring that there are no conflicting
/// constraints sharing a boundary and column index.
/// TODO: generalize the way we store boundary constraints for more trace segments.
#[derive(Default, Debug)]
pub(crate) struct BoundaryStmts {
    boundary_constraints: Vec<(BTreeMap<usize, BoundaryExpr>, BTreeMap<usize, BoundaryExpr>)>,
    variables: Vec<BoundaryVariable>,
}

impl BoundaryStmts {
    // --- ACCESSORS ------------------------------------------------------------------------------

    pub fn num_boundary_constraints(&self, trace_segment: TraceSegment) -> usize {
        self.boundary_constraints[trace_segment as usize].0.len()
            + self.boundary_constraints[trace_segment as usize].1.len()
    }

    pub fn first_boundary_constraints(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<(usize, &BoundaryExpr)> {
        self.boundary_constraints[trace_segment as usize]
            .0
            .iter()
            .map(|(k, v)| (*k, v))
            .collect()
    }

    pub fn last_boundary_constraints(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<(usize, &BoundaryExpr)> {
        self.boundary_constraints[trace_segment as usize]
            .1
            .iter()
            .map(|(k, v)| (*k, v))
            .collect()
    }

    pub fn variables(&self) -> &Vec<BoundaryVariable> {
        &self.variables
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// Add a boundary statement from the AST. The statement can be either be a variable or a
    /// constraint. In case it's a variable, it is added to the variables vector and in case
    /// it is a constraint, it is added to the boundary_constraints vector in the relevant
    /// trace segment.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The identifier specified for the boundary constraint column has not been declared or has
    ///   been declared with the wrong type.
    /// - The constraint expression is contains invalid public input references.
    /// - A boundary constraint has already been declared for the specified column and boundary.
    pub(super) fn insert(
        &mut self,
        symbol_table: &mut SymbolTable,
        stmt: &BoundaryStmt,
    ) -> Result<(), SemanticError> {
        match stmt {
            BoundaryStmt::Variable(boundary_variable) => {
                // validate the expressions inside the variable's values
                match boundary_variable.value() {
                    BoundaryVariableType::Scalar(expr) => validate_expression(symbol_table, expr)?,
                    BoundaryVariableType::Vector(vector) => {
                        for expr in vector {
                            validate_expression(symbol_table, expr)?;
                        }
                    }
                    BoundaryVariableType::Matrix(matrix) => {
                        for row in matrix {
                            for expr in row {
                                validate_expression(symbol_table, expr)?;
                            }
                        }
                    }
                }
                symbol_table.insert_boundary_variable(boundary_variable)?;
                self.variables.push(boundary_variable.clone());
            }
            BoundaryStmt::Constraint(constraint) => {
                // validate the expression
                let expr = constraint.value();
                validate_expression(symbol_table, &expr)?;

                // add the constraint to the specified boundary for the specified trace
                let col_type = symbol_table.get_type(constraint.column())?;
                let result = match col_type {
                    IdentifierType::TraceColumn(column) => match column.trace_segment() {
                        0 => {
                            if self.boundary_constraints.is_empty() {
                                self.boundary_constraints
                                    .push((BTreeMap::default(), BTreeMap::default()));
                            }
                            match constraint.boundary() {
                                ast::Boundary::First => self.boundary_constraints[0]
                                    .0
                                    .insert(column.col_idx(), expr),
                                ast::Boundary::Last => self.boundary_constraints[0]
                                    .1
                                    .insert(column.col_idx(), expr),
                            }
                        }
                        1 => {
                            if self.boundary_constraints.len() == 1 {
                                self.boundary_constraints
                                    .push((BTreeMap::default(), BTreeMap::default()));
                            }
                            match constraint.boundary() {
                                ast::Boundary::First => self.boundary_constraints[1]
                                    .0
                                    .insert(column.col_idx(), expr),
                                ast::Boundary::Last => self.boundary_constraints[1]
                                    .1
                                    .insert(column.col_idx(), expr),
                            }
                        }
                        _ => unimplemented!(),
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
            }
        }

        Ok(())
    }
}

/// Recursively validates the BoundaryExpression.
///
/// # Errors
/// Returns an error if the expression includes a reference to a public input that hasn't been
/// declared or to an invalid index in an existing public input.
/// TODO: Complete implementation of validation.
fn validate_expression(
    symbol_table: &SymbolTable,
    expr: &ast::BoundaryExpr,
) -> Result<(), SemanticError> {
    match expr {
        BoundaryExpr::Elem(ident) => {
            symbol_table.get_type(ident.name())?;
            Ok(())
        }
        BoundaryExpr::VectorAccess(vector_access) => {
            symbol_table.access_vector_element(vector_access)?;
            Ok(())
        }
        BoundaryExpr::MatrixAccess(matrix_access) => {
            symbol_table.access_matrix_element(matrix_access)?;
            Ok(())
        }
        BoundaryExpr::Add(lhs, rhs) | BoundaryExpr::Sub(lhs, rhs) => {
            validate_expression(symbol_table, lhs)?;
            validate_expression(symbol_table, rhs)
        }
        _ => Ok(()),
    }
}
