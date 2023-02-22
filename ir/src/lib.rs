pub use air_script_core::{
    Constant, ConstantType, Expression, Identifier, IndexedTraceAccess, MatrixAccess,
    NamedTraceAccess, TraceSegment, Variable, VariableType, VectorAccess,
};
pub use parser::ast::{self, Boundary, BoundaryStmt, IntegrityStmt, PublicInput};
use std::collections::BTreeMap;

mod symbol_table;
use symbol_table::{Scope, SymbolTable};

pub mod constraints;
use constraints::{AlgebraicGraph, ConstraintRoot, Constraints, MIN_CYCLE_LENGTH};
pub use constraints::{IntegrityConstraintDegree, NodeIndex};

mod trace_columns;

mod error;
use error::SemanticError;

mod helpers;
use helpers::SourceValidator;

#[cfg(test)]
mod tests;

// TYPE ALIASES
// ================================================================================================
pub type Constants = Vec<Constant>;
pub type PublicInputs = Vec<(String, usize)>;
pub type PeriodicColumns = Vec<Vec<u64>>;
pub type BoundaryConstraintsMap = BTreeMap<usize, Expression>;

// AIR IR
// ================================================================================================

/// Internal representation of an AIR.
///
/// TODO: docs
#[derive(Default, Debug)]
pub struct AirIR {
    air_name: String,
    segment_widths: Vec<u16>,
    constants: Constants,
    public_inputs: PublicInputs,
    periodic_columns: PeriodicColumns,
    constraints: Constraints,
}

impl AirIR {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    /// Consumes the provided source and generates a matching AirIR.
    pub fn new(source: &ast::Source) -> Result<Self, SemanticError> {
        let ast::Source(source) = source;

        // set a default name.
        let mut air_name = "CustomAir";

        let mut validator = SourceValidator::new();

        // process the declarations of identifiers first, using a single symbol table to enforce
        // uniqueness.
        let mut symbol_table = SymbolTable::default();

        for section in source {
            match section {
                ast::SourceSection::AirDef(Identifier(air_def)) => {
                    // update the name of the air.
                    air_name = air_def;
                }
                ast::SourceSection::Constant(constant) => {
                    symbol_table.insert_constant(constant)?;
                }
                ast::SourceSection::Trace(columns) => {
                    // process & validate the main trace columns
                    symbol_table.insert_trace_columns(0, &columns.main_cols)?;
                    validator.exists("main_trace_columns");
                    if !columns.aux_cols.is_empty() {
                        // process & validate the auxiliary trace columns
                        symbol_table.insert_trace_columns(1, &columns.aux_cols)?;
                        validator.exists("aux_trace_columns");
                    }
                }
                ast::SourceSection::PublicInputs(inputs) => {
                    // process & validate the public inputs
                    symbol_table.insert_public_inputs(inputs)?;
                    validator.exists("public_inputs");
                }
                ast::SourceSection::PeriodicColumns(columns) => {
                    // process & validate the periodic columns
                    symbol_table.insert_periodic_columns(columns)?;
                }
                ast::SourceSection::RandomValues(values) => {
                    symbol_table.insert_random_values(values)?;
                    validator.exists("random_values");
                }
                _ => {}
            }
        }

        // then process the constraints & validate them against the symbol table.
        let num_trace_segments = symbol_table.num_trace_segments();
        let mut constraints = Constraints::new(num_trace_segments);

        for section in source {
            match section {
                ast::SourceSection::BoundaryConstraints(stmts) => {
                    for stmt in stmts {
                        constraints.insert_boundary_stmt(&mut symbol_table, stmt)?
                    }
                    validator.exists("boundary_constraints");
                }
                ast::SourceSection::IntegrityConstraints(stmts) => {
                    for stmt in stmts {
                        constraints.insert_integrity_stmt(&mut symbol_table, stmt)?
                    }
                    validator.exists("integrity_constraints");
                }
                _ => {}
            }
        }

        let (segment_widths, constants, public_inputs, periodic_columns) =
            symbol_table.into_declarations();

        // validate sections
        validator.check()?;

        Ok(Self {
            air_name: air_name.to_string(),
            segment_widths,
            constants,
            public_inputs,
            periodic_columns,
            constraints,
        })
    }

    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    pub fn air_name(&self) -> &str {
        &self.air_name
    }

    pub fn constants(&self) -> &Constants {
        &self.constants
    }

    pub fn segment_widths(&self) -> &Vec<u16> {
        &self.segment_widths
    }

    pub fn public_inputs(&self) -> &PublicInputs {
        &self.public_inputs
    }

    pub fn periodic_columns(&self) -> &PeriodicColumns {
        &self.periodic_columns
    }

    // --- PUBLIC ACCESSORS FOR BOUNDARY CONSTRAINTS ----------------------------------------------

    pub fn num_boundary_constraints(&self, trace_segment: u8) -> usize {
        self.constraints.num_boundary_constraints(trace_segment)
    }

    pub fn boundary_constraints(&self, trace_segment: TraceSegment) -> &[ConstraintRoot] {
        self.constraints.boundary_constraints(trace_segment)
    }

    // --- PUBLIC ACCESSORS FOR INTEGRITY CONSTRAINTS ---------------------------------------------

    pub fn validity_constraint_degrees(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<IntegrityConstraintDegree> {
        self.constraints.validity_constraint_degrees(trace_segment)
    }

    pub fn validity_constraints(&self, trace_segment: TraceSegment) -> &[ConstraintRoot] {
        self.constraints.validity_constraints(trace_segment)
    }

    pub fn transition_constraint_degrees(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<IntegrityConstraintDegree> {
        self.constraints
            .transition_constraint_degrees(trace_segment)
    }

    pub fn transition_constraints(&self, trace_segment: TraceSegment) -> &[ConstraintRoot] {
        self.constraints.transition_constraints(trace_segment)
    }

    pub fn constraint_graph(&self) -> &AlgebraicGraph {
        self.constraints.graph()
    }
}
