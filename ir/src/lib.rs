pub use parser::ast::{
    self, boundary_constraints::BoundaryExpr, constants::Constant, Identifier, IntegrityVariable,
    PublicInput,
};
use std::collections::BTreeMap;

mod symbol_table;
use symbol_table::{IdentifierType, SymbolTable};

pub mod boundary_stmts;
use boundary_stmts::BoundaryStmts;

pub mod integrity_stmts;
use integrity_stmts::{
    AlgebraicGraph, ConstraintRoot, IntegrityStmts, VariableValue, MIN_CYCLE_LENGTH,
};
pub use integrity_stmts::{IntegrityConstraintDegree, NodeIndex};

mod trace_columns;

mod error;
use error::SemanticError;

mod helpers;
use helpers::SourceValidator;

#[cfg(test)]
mod tests;

// ==== ALIASES ===================================================================================
pub type TraceSegment = u8;
pub type Constants = Vec<Constant>;
pub type PublicInputs = Vec<(String, usize)>;
pub type PeriodicColumns = Vec<Vec<u64>>;
pub type BoundaryConstraintsMap = BTreeMap<usize, BoundaryExpr>;
pub type VariableRoots = BTreeMap<VariableValue, (TraceSegment, NodeIndex)>;

// ==== CONSTANTS =================================================================================
const CURRENT_ROW: usize = 0;
const NEXT_ROW: usize = 1;

// ==== AIR IR ====================================================================================

/// Internal representation of an AIR.
///
/// TODO: docs
#[derive(Default, Debug)]
pub struct AirIR {
    air_name: String,
    constants: Constants,
    public_inputs: PublicInputs,
    periodic_columns: PeriodicColumns,
    boundary_stmts: BoundaryStmts,
    integrity_stmts: IntegrityStmts,
}

impl AirIR {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    /// Consumes the provided source and generates a matching AirIR.
    pub fn from_source(source: &ast::Source) -> Result<Self, SemanticError> {
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
                    if !columns.aux_cols.is_empty() {
                        // process & validate the auxiliary trace columns
                        symbol_table.insert_trace_columns(1, &columns.aux_cols)?;
                    }
                    validator.exists("trace_columns");
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
                _ => {}
            }
        }

        let num_trace_segments = symbol_table.num_trace_segments();
        // then process the constraints & validate them against the symbol table.
        let mut boundary_stmts = BoundaryStmts::new(num_trace_segments);
        let mut integrity_stmts = IntegrityStmts::new(num_trace_segments);
        for section in source {
            match section {
                ast::SourceSection::BoundaryConstraints(stmts) => {
                    for stmt in stmts {
                        boundary_stmts.insert(&symbol_table, stmt)?
                    }
                    validator.exists("boundary_constraints");
                }
                ast::SourceSection::IntegrityConstraints(stmts) => {
                    for stmt in stmts {
                        integrity_stmts.insert(&mut symbol_table, stmt)?
                    }
                    validator.exists("integrity_constraints");
                }
                _ => {}
            }
        }

        let (constants, public_inputs, periodic_columns) = symbol_table.into_declarations();

        // validate sections
        validator.check()?;

        Ok(Self {
            air_name: air_name.to_string(),
            constants,
            public_inputs,
            periodic_columns,
            boundary_stmts,
            integrity_stmts,
        })
    }

    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    pub fn air_name(&self) -> &str {
        &self.air_name
    }

    pub fn constants(&self) -> &Constants {
        &self.constants
    }

    pub fn public_inputs(&self) -> &PublicInputs {
        &self.public_inputs
    }

    pub fn periodic_columns(&self) -> &PeriodicColumns {
        &self.periodic_columns
    }

    // --- PUBLIC ACCESSORS FOR BOUNDARY CONSTRAINTS ----------------------------------------------

    pub fn num_main_assertions(&self) -> usize {
        self.boundary_stmts.num_boundary_constraints(0)
    }

    pub fn main_first_boundary_constraints(&self) -> Vec<(usize, &BoundaryExpr)> {
        self.boundary_stmts.first_boundary_constraints(0)
    }

    pub fn main_last_boundary_constraints(&self) -> Vec<(usize, &BoundaryExpr)> {
        self.boundary_stmts.last_boundary_constraints(0)
    }

    pub fn num_aux_assertions(&self) -> usize {
        self.boundary_stmts.num_boundary_constraints(1)
    }

    pub fn aux_first_boundary_constraints(&self) -> Vec<(usize, &BoundaryExpr)> {
        self.boundary_stmts.first_boundary_constraints(1)
    }

    pub fn aux_last_boundary_constraints(&self) -> Vec<(usize, &BoundaryExpr)> {
        self.boundary_stmts.last_boundary_constraints(1)
    }

    // --- PUBLIC ACCESSORS FOR INTEGRITY CONSTRAINTS --------------------------------------------

    pub fn constraint_degrees(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<IntegrityConstraintDegree> {
        self.integrity_stmts.constraint_degrees(trace_segment)
    }

    pub fn integrity_constraints(&self, trace_segment: TraceSegment) -> &[ConstraintRoot] {
        self.integrity_stmts.constraints(trace_segment)
    }

    pub fn constraint_graph(&self) -> &AlgebraicGraph {
        self.integrity_stmts.graph()
    }
}
