pub use air_script_core::{
    AccessType, ConstantBinding, ConstantValueExpr, Expression, Identifier, Iterable,
    ListComprehension, ListFolding, ListFoldingValueExpr, SymbolAccess, TraceAccess, TraceBinding,
    TraceSegment, VariableBinding, VariableValueExpr,
};
pub use parser::ast;
use std::collections::{BTreeMap, BTreeSet};

pub mod constraint_builder;
use constraint_builder::{ConstrainedBoundary, ConstraintBuilder, Evaluator};

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
pub use symbol_table::Value;
use symbol_table::{Symbol, SymbolBinding, SymbolTable};

mod validation;
use validation::{Section, SemanticError, SourceValidator};

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
        let mut ev_decls = Vec::new();
        let mut pub_input_decls = Vec::new();
        let mut trace_decls = Vec::new();

        let mut boundary_stmts = Vec::new();
        let mut integrity_stmts = Vec::new();

        let mut validator = SourceValidator::new();
        for section in source {
            match section {
                ast::SourceSection::AirDef(Identifier(air_def)) => {
                    // update the name of the air.
                    air_name = air_def;
                }
                ast::SourceSection::Constant(constant) => {
                    // process & validate the constant.
                    symbol_table.insert_constant(constant)?;
                }
                ast::SourceSection::Trace(trace_bindings) => {
                    if !trace_bindings.is_empty() {
                        validator.exists(Section::MainTraceColumns);
                    }
                    if trace_bindings.len() > 1 {
                        validator.exists(Section::AuxTraceColumns);
                    }
                    // accumulate and save the trace bindings for later processing.
                    trace_decls.push(trace_bindings);
                }
                ast::SourceSection::PublicInputs(inputs) => {
                    validator.exists(Section::PublicInputs);
                    // accumulate and save the public input bindings for later processing.
                    pub_input_decls.extend(inputs);
                }
                ast::SourceSection::PeriodicColumns(columns) => {
                    // process & validate the periodic columns
                    symbol_table.insert_periodic_columns(columns)?;
                }
                ast::SourceSection::RandomValues(values) => {
                    validator.exists(Section::RandomValues);
                    // process & validate the random value declarations
                    symbol_table.insert_random_values(values)?;
                }
                ast::SourceSection::BoundaryConstraints(stmts) => {
                    validator.exists(Section::BoundaryConstraints);
                    // save the boundary statements for processing after the SymbolTable is built.
                    boundary_stmts.extend(stmts);
                }
                ast::SourceSection::IntegrityConstraints(stmts) => {
                    validator.exists(Section::IntegrityConstraints);
                    // save the integrity statements for processing after the SymbolTable is built.
                    integrity_stmts.extend(stmts);
                }
                ast::SourceSection::EvaluatorFunction(ev_expr) => ev_decls.push(ev_expr),
            }
        }

        // validate sections
        validator.check()?;

        // process evaluators
        let mut evaluators: BTreeMap<String, Evaluator> = BTreeMap::new();
        for ev_decl in ev_decls {
            let constraint_builder = ConstraintBuilder::new(symbol_table.clone(), evaluators);
            evaluators = constraint_builder.process_evaluator(ev_decl)?;
        }

        // process & validate the public inputs
        symbol_table.insert_public_inputs(pub_input_decls)?;
        // process & validate the trace bindings
        for trace_bindings in trace_decls.into_iter() {
            symbol_table.insert_trace_bindings(trace_bindings)?;
        }

        // process the variable & constraint statements, and validate them against the symbol table.
        let mut constraint_builder = ConstraintBuilder::new(symbol_table, evaluators);
        constraint_builder.insert_constraints(boundary_stmts, integrity_stmts)?;

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

    pub fn constants(&self) -> &[ConstantBinding] {
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

    pub fn integrity_constraints(&self, trace_segment: TraceSegment) -> &[ConstraintRoot] {
        self.constraints.integrity_constraints(trace_segment)
    }

    pub fn integrity_constraint_degrees(
        &self,
        trace_segment: TraceSegment,
    ) -> Vec<IntegrityConstraintDegree> {
        self.constraints.integrity_constraint_degrees(trace_segment)
    }

    pub fn validity_constraints(&self, trace_segment: TraceSegment) -> Vec<&ConstraintRoot> {
        self.constraints
            .integrity_constraints(trace_segment)
            .iter()
            .filter(|constraint| matches!(constraint.domain(), ConstraintDomain::EveryRow))
            .collect()
    }

    pub fn transition_constraints(&self, trace_segment: TraceSegment) -> Vec<&ConstraintRoot> {
        self.constraints
            .integrity_constraints(trace_segment)
            .iter()
            .filter(|constraint| matches!(constraint.domain(), ConstraintDomain::EveryFrame(_)))
            .collect()
    }

    pub fn constraint_graph(&self) -> &AlgebraicGraph {
        self.constraints.graph()
    }
}
