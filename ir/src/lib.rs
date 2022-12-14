use parser::ast;
pub use parser::ast::{boundary_constraints::BoundaryExpr, Identifier, PublicInput};
use std::collections::BTreeMap;

mod symbol_table;
use symbol_table::{IdentifierType, SymbolTable};

pub mod boundary_constraints;
use boundary_constraints::BoundaryConstraints;

pub mod transition_constraints;
use transition_constraints::{AlgebraicGraph, TransitionConstraints, MIN_CYCLE_LENGTH};
pub use transition_constraints::{NodeIndex, TransitionConstraintDegree};

mod error;
use error::SemanticError;

mod helpers;
use helpers::SourceValidator;

pub type PublicInputs = Vec<(String, usize)>;
pub type PeriodicColumns = Vec<Vec<u64>>;

/// Internal representation of an AIR.
///
/// TODO: docs
#[derive(Default, Debug)]
pub struct AirIR {
    air_name: String,
    public_inputs: PublicInputs,
    periodic_columns: PeriodicColumns,
    boundary_constraints: BoundaryConstraints,
    transition_constraints: TransitionConstraints,
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
                ast::SourceSection::TraceCols(columns) => {
                    // process & validate the main trace columns
                    symbol_table.insert_main_trace_columns(&columns.main_cols)?;
                    // process & validate the auxiliary trace columns
                    symbol_table.insert_aux_trace_columns(&columns.aux_cols)?;
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

        // then process the constraints & validate them against the symbol table.
        let mut boundary_constraints = BoundaryConstraints::default();
        let mut transition_constraints = TransitionConstraints::default();
        for section in source {
            match section {
                ast::SourceSection::BoundaryConstraints(constraints) => {
                    for constraint in constraints.boundary_constraints.iter() {
                        boundary_constraints.insert(&symbol_table, constraint)?;
                    }
                    validator.exists("boundary_constraints");
                }
                ast::SourceSection::TransitionConstraints(constraints) => {
                    for constraint in constraints.transition_constraints.iter() {
                        transition_constraints.insert(&symbol_table, constraint)?;
                    }
                    validator.exists("transition_constraints");
                }
                _ => {}
            }
        }

        let (public_inputs, periodic_columns) = symbol_table.into_declarations();

        // validate sections
        validator.check()?;

        Ok(Self {
            air_name: air_name.to_string(),
            public_inputs,
            periodic_columns,
            boundary_constraints,
            transition_constraints,
        })
    }

    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    pub fn air_name(&self) -> &str {
        &self.air_name
    }

    pub fn public_inputs(&self) -> &PublicInputs {
        &self.public_inputs
    }

    pub fn periodic_columns(&self) -> &PeriodicColumns {
        &self.periodic_columns
    }

    // --- PUBLIC ACCESSORS FOR BOUNDARY CONSTRAINTS ----------------------------------------------

    pub fn num_main_assertions(&self) -> usize {
        self.boundary_constraints.main_len()
    }

    pub fn main_first_boundary_constraints(&self) -> Vec<(usize, &BoundaryExpr)> {
        self.boundary_constraints.main_first()
    }

    pub fn main_last_boundary_constraints(&self) -> Vec<(usize, &BoundaryExpr)> {
        self.boundary_constraints.main_last()
    }

    pub fn num_aux_assertions(&self) -> usize {
        self.boundary_constraints.aux_len()
    }

    pub fn aux_first_boundary_constraints(&self) -> Vec<(usize, &BoundaryExpr)> {
        self.boundary_constraints.aux_first()
    }

    pub fn aux_last_boundary_constraints(&self) -> Vec<(usize, &BoundaryExpr)> {
        self.boundary_constraints.aux_last()
    }

    // --- PUBLIC ACCESSORS FOR TRANSITION CONSTRAINTS --------------------------------------------

    pub fn main_degrees(&self) -> Vec<TransitionConstraintDegree> {
        self.transition_constraints.main_degrees()
    }

    pub fn main_transition_constraints(&self) -> &[NodeIndex] {
        self.transition_constraints.main_constraints()
    }

    pub fn aux_degrees(&self) -> Vec<TransitionConstraintDegree> {
        self.transition_constraints.aux_degrees()
    }

    pub fn aux_transition_constraints(&self) -> &[NodeIndex] {
        self.transition_constraints.aux_constraints()
    }

    pub fn transition_graph(&self) -> &AlgebraicGraph {
        self.transition_constraints.graph()
    }
}

// TODO: add checks for the correctness of the AirIR that is built.
#[cfg(test)]
mod tests {
    use super::*;
    use parser::parse;

    #[test]
    fn boundary_constraints() {
        let source = "
        trace_columns:
            main: [clk]
        public_inputs:
            stack_inputs: [16]
        transition_constraints:
            enf clk' = clk + 1
        boundary_constraints:
            enf clk.first = 0
            enf clk.last = 1";

        let parsed = parse(source).expect("Parsing failed");
        let result = AirIR::from_source(&parsed);
        assert!(result.is_ok());
    }

    #[test]
    fn err_bc_column_undeclared() {
        let source = "
        trace_columns:
            main: [ctx]
        public_inputs:
            stack_inputs: [16]
        boundary_constraints:
            enf clk.first = 0
            enf clk.last = 1
        transition_constraints:
            enf clk' = clk + 1";

        let parsed = parse(source).expect("Parsing failed");

        let result = AirIR::from_source(&parsed);
        assert!(result.is_err());
    }

    #[test]
    fn err_bc_empty_or_omitted() {
        // if boundary constraints are empty, an error should be returned at parser level.
        let source = "
        trace_columns:
            main: [clk]
        public_inputs:
            stack_inputs: [16]
        boundary_constraints:
        transition_constraints:
            enf clk' = clk + 1";

        assert!(parse(source).is_err());

        // if boundary constraints are omitted, an error should be returned at IR level.
        let source = "
        trace_columns:
            main: [clk]
        public_inputs:
            stack_inputs: [16]
        transition_constraints:
            enf clk' = clk + 1";

        let parsed = parse(source).expect("Parsing failed");
        let result = AirIR::from_source(&parsed);
        assert!(result.is_err());
    }

    #[test]
    fn err_bc_duplicate_first() {
        let source = "
        trace_columns:
            main: [clk]
        public_inputs:
            stack_inputs: [16]
        transition_constraints:
            enf clk' = clk + 1
        boundary_constraints:
            enf clk.first = 0
            enf clk.first = 1";

        let parsed = parse(source).expect("Parsing failed");
        let result = AirIR::from_source(&parsed);

        assert!(result.is_err());
    }

    #[test]
    fn err_bc_duplicate_last() {
        let source = "
        trace_columns:
            main: [clk]
        public_inputs:
            stack_inputs: [16]
        transition_constraints:
            enf clk' = clk + 1
        boundary_constraints:
            enf clk.last = 0
            enf clk.last = 1";

        let parsed = parse(source).expect("Parsing failed");

        assert!(AirIR::from_source(&parsed).is_err());
    }

    #[test]
    fn transition_constraints() {
        let source = "
        trace_columns:
            main: [clk]
        public_inputs:
            stack_inputs: [16]
        boundary_constraints:
            enf clk.first = 0
        transition_constraints:
            enf clk' = clk + 1";

        let parsed = parse(source).expect("Parsing failed");

        let result = AirIR::from_source(&parsed);
        assert!(result.is_ok());
    }

    #[test]
    fn transition_constraints_using_parens() {
        let source = "
        trace_columns:
            main: [clk]
        public_inputs:
            stack_inputs: [16]
        boundary_constraints:
            enf clk.first = 0
        transition_constraints:
            enf clk' = (clk + 1)";

        let parsed = parse(source).expect("Parsing failed");

        let result = AirIR::from_source(&parsed);
        assert!(result.is_ok());
    }

    #[test]
    fn err_tc_empty_or_omitted() {
        // if transition constraints are empty, an error should be returned at parser level.
        let source = "
        trace_columns:
            main: [clk]
        public_inputs:
            stack_inputs: [16]
        transition_constraints:
        boundary_constraints:
            enf clk.first = 0";

        assert!(parse(source).is_err());

        // if transition constraints are omitted, an error should be returned at IR level.
        let source = "
        trace_columns:
            main: [clk]
        public_inputs:
            stack_inputs: [16]
        boundary_constraints:
            enf clk.first = 0";

        let parsed = parse(source).expect("Parsing failed");
        let result = AirIR::from_source(&parsed);
        assert!(result.is_err());
    }

    #[test]
    fn err_tc_column_undeclared() {
        let source = "
        trace_columns:
            main: [ctx]
        public_inputs:
            stack_inputs: [16]
        boundary_constraints:
            enf ctx.first = 0
        transition_constraints:
            enf clk' = clk + 1";

        let parsed = parse(source).expect("Parsing failed");

        let result = AirIR::from_source(&parsed);
        assert!(result.is_err());
    }

    #[test]
    fn err_public_inputs_empty_or_omitted() {
        // if public inputs are empty, an error should be returned at parser level.
        let source = "
        trace_columns:
            main: [clk]
        public_inputs:
        transition_constraints:
            enf clk' = clk + 1
        boundary_constraints:
            enf clk.first = 0";

        assert!(parse(source).is_err());

        // if public inputs are omitted, an error should be returned at IR level.
        let source = "
        trace_columns:
            main: [clk]
        transition_constraints:
            enf clk' = clk + 1
        boundary_constraints:
            enf clk.first = 0";

        let parsed = parse(source).expect("Parsing failed");
        let result = AirIR::from_source(&parsed);
        assert!(result.is_err());
    }

    #[test]
    fn err_trace_cols_omitted() {
        // returns an error if trace columns declaration is missing
        let source = "
        public_inputs:
            stack_inputs: [16]
        transition_constraints:
            enf clk' = clk + 1
        boundary_constraints:
            enf clk.first = 0";

        let parsed = parse(source).expect("Parsing failed");

        let result = AirIR::from_source(&parsed);

        // this fails before the check for missing trace columns declaration since the clk column
        // used in constraints is not declared.
        assert!(result.is_err());
    }

    #[test]
    fn op_mul() {
        let source = "
        trace_columns:
            main: [clk]
        public_inputs:
            stack_inputs: [16]
        boundary_constraints:
            enf clk.first = 0
        transition_constraints:
            enf clk' * clk = 1";
        let parsed = parse(source).expect("Parsing failed");

        let result = AirIR::from_source(&parsed);
        assert!(result.is_ok());
    }

    #[test]
    fn op_exp() {
        let source = "
        trace_columns:
            main: [clk]
        public_inputs:
            stack_inputs: [16]
        boundary_constraints:
            enf clk.first = 0
        transition_constraints:
            enf clk'^2 - clk = 1";
        let parsed = parse(source).expect("Parsing failed");

        let result = AirIR::from_source(&parsed);
        assert!(result.is_ok());
    }
}
