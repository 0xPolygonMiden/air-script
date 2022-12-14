use crate::{BoundaryConstraintsMap, TraceSegment};

use super::{BTreeMap, BoundaryExpr, IdentifierType, SemanticError, SymbolTable};
use parser::ast::{self, BoundaryStmt};

// BOUNDARY CONSTRAINTS
// ================================================================================================

/// A struct containing all of the boundary constraints to be applied at each of the 2 allowed
/// boundaries (first row and last row). The constraints are stored in a vector of
/// [SegmentBoundaryConstraints] where each index of the vector corresponds to the trace segment
/// of the column the boundary constraint applies to. For ease of code generation and evaluation,
/// constraints are sorted into maps by the boundary. This also simplifies ensuring that there are
/// no conflicting constraints sharing a boundary and column index.
#[derive(Default, Debug)]
pub(crate) struct BoundaryStmts {
    boundary_constraints: Vec<SegmentBoundaryConstraints>,
}

/// A struct containing boundary constraints applied to columns in a segment at the first and last
/// rows.
#[derive(Default, Debug, Clone)]
struct SegmentBoundaryConstraints {
    first: BoundaryConstraintsMap,
    last: BoundaryConstraintsMap,
}

impl SegmentBoundaryConstraints {
    fn new() -> Self {
        Self {
            first: BTreeMap::new(),
            last: BTreeMap::new(),
        }
    }

    fn first(&self) -> &BoundaryConstraintsMap {
        &self.first
    }

    fn last(&self) -> &BoundaryConstraintsMap {
        &self.last
    }

    fn first_mut(&mut self) -> &mut BoundaryConstraintsMap {
        &mut self.first
    }

    fn last_mut(&mut self) -> &mut BoundaryConstraintsMap {
        &mut self.last
    }
}

impl BoundaryStmts {
    /// Creates a new [BoundaryStmts] instance with the specified number of trace segments.
    pub fn new(num_trace_segments: usize) -> Self {
        Self {
            boundary_constraints: vec![SegmentBoundaryConstraints::new(); num_trace_segments],
        }
    }

    // --- ACCESSORS ------------------------------------------------------------------------------

    /// Returns the number of boundary constraints for the specified trace segment.
    pub fn num_boundary_constraints(&self, trace_segment: TraceSegment) -> usize {
        let trace_segment = trace_segment as usize;
        if self.boundary_constraints.len() <= trace_segment {
            0
        } else {
            self.boundary_constraints[trace_segment].first.len()
                + self.boundary_constraints[trace_segment].last.len()
        }
    }

    /// Returns a vector of the boundary constraints for the specified trace segment at the
    /// first row.
    pub fn first_boundary_constraints(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<(usize, &BoundaryExpr)> {
        if self.boundary_constraints.len() <= trace_segment.into() {
            Vec::new()
        } else {
            self.boundary_constraints[trace_segment as usize]
                .first()
                .iter()
                .map(|(k, v)| (*k, v))
                .collect()
        }
    }

    /// Returns a vector of the boundary constraints for the specified trace segment at the
    /// last row.
    pub fn last_boundary_constraints(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<(usize, &BoundaryExpr)> {
        if self.boundary_constraints.len() <= trace_segment.into() {
            Vec::new()
        } else {
            self.boundary_constraints[trace_segment as usize]
                .last()
                .iter()
                .map(|(k, v)| (*k, v))
                .collect()
        }
    }

    // --- MUTATORS -------------------------------------------------------------------------------

    /// Adds a boundary statement from the AST. The statement can either be a variable or a
    /// constraint. In case it is a constraint, it is added to the boundary_constraints vector
    /// in the relevant trace segment. Variables are currently not supported for boundary
    /// constraints.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The identifier specified for the boundary constraint column has not been declared or has
    ///   been declared with the wrong type.
    /// - The constraint expression contains invalid public input references.
    /// - A boundary constraint has already been declared for the specified column and boundary.
    pub(super) fn insert(
        &mut self,
        symbol_table: &SymbolTable,
        stmt: &BoundaryStmt,
    ) -> Result<(), SemanticError> {
        match stmt {
            BoundaryStmt::Variable(_) => unimplemented!(),
            BoundaryStmt::Constraint(constraint) => {
                // validate the expression
                let expr = constraint.value();
                validate_expression(symbol_table, &expr)?;

                // add the constraint to the specified boundary for the specified trace
                let col_type = symbol_table.get_type(constraint.column().name())?;
                let result = match col_type {
                    IdentifierType::TraceColumn(column) => match constraint.boundary() {
                        ast::Boundary::First => self.boundary_constraints
                            [column.trace_segment() as usize]
                            .first_mut()
                            .insert(column.col_idx(), expr),
                        ast::Boundary::Last => self.boundary_constraints
                            [column.trace_segment() as usize]
                            .last_mut()
                            .insert(column.col_idx(), expr),
                    },
                    _ => {
                        return Err(SemanticError::InvalidUsage(format!(
                            "Identifier {} was declared as a {}, not as a trace column",
                            constraint.column().name(),
                            col_type
                        )));
                    }
                };

                // raise an error if multiple constraints were applied to the same boundary
                if result.is_some() {
                    return Err(SemanticError::TooManyConstraints(format!(
                        "A boundary constraint was already defined for {} '{}' at the {}",
                        col_type,
                        constraint.column().name(),
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
        BoundaryExpr::Add(lhs, rhs) | BoundaryExpr::Sub(lhs, rhs) | BoundaryExpr::Mul(lhs, rhs) => {
            validate_expression(symbol_table, lhs)?;
            validate_expression(symbol_table, rhs)
        }
        BoundaryExpr::Exp(lhs, _) => validate_expression(symbol_table, lhs),
        _ => Ok(()),
    }
}
