pub use air_script_core::{
    Constant, ConstantType, Expression, Identifier, IndexedTraceAccess, Iterable,
    ListComprehension, ListFoldingType, ListFoldingValueType, MatrixAccess, NamedTraceAccess,
    TraceSegment, Variable, VariableType, VectorAccess,
};
pub use parser::ast;
use std::collections::{BTreeMap, BTreeSet};

pub mod constraint_builder;
use constraint_builder::{ConstrainedBoundary, ConstraintBuilder};

pub mod constraints;
use constraints::{
    AlgebraicGraph, ConstraintDomain, ConstraintRoot, Constraints, Operation, CURRENT_ROW,
    MIN_CYCLE_LENGTH,
};
pub use constraints::{IntegrityConstraintDegree, NodeIndex};

pub mod declarations;
use declarations::Declarations;
pub use declarations::{PeriodicColumn, PublicInput};

mod symbol_table;
use symbol_table::{AccessType, Symbol, SymbolTable, SymbolType, ValidateAccess};
pub use symbol_table::{ConstantValue, Value};

mod validation;
use validation::{SemanticError, SourceValidator};

#[cfg(test)]
mod tests;

// TYPE ALIASES
// ================================================================================================
pub type BoundaryConstraintsMap = BTreeMap<usize, Expression>;

// AIR IR
// ================================================================================================

/// Internal representation of an AIR.
///
/// TODO: docs
#[derive(Default, Debug)]
pub struct AirIR {
    air_name: String,
    declarations: Declarations,
    constraints: Constraints,
}

impl AirIR {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    /// Consumes the provided source and generates a matching AirIR.
    pub fn new(source: ast::Source) -> Result<Self, SemanticError> {
        let ast::Source(source) = source;

        // set a default name.
        let mut air_name = String::from("CustomAir");

        // process the declarations of identifiers first, using a single symbol table to enforce
        // uniqueness.
        let mut symbol_table = SymbolTable::default();
        let mut validator = SourceValidator::new();
        let mut boundary_stmts = Vec::new();
        let mut integrity_stmts = Vec::new();

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
                ast::SourceSection::BoundaryConstraints(stmts) => {
                    // save the boundary statements for processing after the SymbolTable is built.
                    boundary_stmts.extend(stmts);
                    validator.exists("boundary_constraints");
                }
                ast::SourceSection::IntegrityConstraints(stmts) => {
                    // save the integrity statements for processing after the SymbolTable is built.
                    integrity_stmts.extend(stmts);
                    validator.exists("integrity_constraints");
                }
            }
        }

        // validate sections
        validator.check()?;

        // process the variable & constraint statements, and validate them against the symbol table.
        let mut constraint_builder = ConstraintBuilder::new(symbol_table);
        constraint_builder.insert_boundary_constraints(boundary_stmts)?;
        constraint_builder.insert_integrity_constraints(integrity_stmts)?;

        let (declarations, constraints) = constraint_builder.into_air();

        Ok(Self {
            air_name,
            declarations,
            constraints,
        })
    }

    // --- PUBLIC ACCESSORS FOR DECLARATIONS ------------------------------------------------------

    pub fn air_name(&self) -> &str {
        &self.air_name
    }

    pub fn constants(&self) -> &[Constant] {
        self.declarations.constants()
    }

    pub fn periodic_columns(&self) -> &[PeriodicColumn] {
        self.declarations.periodic_columns()
    }

    pub fn public_inputs(&self) -> &[PublicInput] {
        self.declarations.public_inputs()
    }

    pub fn trace_segment_widths(&self) -> &[u16] {
        self.declarations.trace_segment_widths()
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
