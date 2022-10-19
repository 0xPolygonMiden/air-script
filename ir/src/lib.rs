use parser::ast;
pub use parser::ast::{Expr, Identifier};

mod trace_columns;
use trace_columns::TraceColumns;

pub mod boundary_constraints;
use boundary_constraints::BoundaryConstraints;

pub mod transition_constraints;
use transition_constraints::{AlgebraicGraph, NodeIndex, TransitionConstraints};

mod error;
use error::SemanticError;

/// Internal representation of an AIR.
///
/// TODO: periodic columns, public inputs, random values, and auxiliary trace constraints
#[derive(Default, Debug)]
pub struct AirIR {
    air_name: String,
    main_boundary_constraints: BoundaryConstraints,
    main_transition_constraints: TransitionConstraints,
}

impl AirIR {
    // --- CONSTRUCTOR ----------------------------------------------------------------------------

    /// Consumes the provided source and generates a matching AirIR.
    pub fn from_source(source: &ast::Source) -> Result<Self, SemanticError> {
        let ast::Source(source) = source;
        // set a default name.
        let mut air_name = "CustomAir";

        // process the trace columns first.
        let mut trace_columns = TraceColumns::default();
        for section in source {
            // TODO: each of these sections should only exist once in the AST
            match section {
                ast::SourceSection::AirDef(Identifier(air_def)) => {
                    air_name = air_def;
                }
                ast::SourceSection::TraceCols(columns) => {
                    for (idx, Identifier(name)) in columns.main_cols.iter().enumerate() {
                        trace_columns.insert(name, idx)?;
                    }
                }
                _ => {}
            }
        }

        // then process the constraints.
        let mut main_boundary_constraints = BoundaryConstraints::default();
        let mut main_transition_constraints = TransitionConstraints::default();
        for section in source {
            match section {
                ast::SourceSection::BoundaryConstraints(constraints) => {
                    for constraint in constraints.boundary_constraints.iter() {
                        main_boundary_constraints.insert(constraint, &trace_columns)?;
                    }
                }
                ast::SourceSection::TransitionConstraints(constraints) => {
                    for constraint in constraints.transition_constraints.iter() {
                        main_transition_constraints.insert(constraint, &trace_columns)?;
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            air_name: air_name.to_string(),
            main_boundary_constraints,
            main_transition_constraints,
        })
    }

    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------
    pub fn air_name(&self) -> &str {
        &self.air_name
    }

    pub fn num_main_assertions(&self) -> usize {
        self.main_boundary_constraints.len()
    }

    pub fn main_first_boundary_constraints(&self) -> Vec<&Expr> {
        self.main_boundary_constraints.first()
    }

    pub fn main_last_boundary_constraints(&self) -> Vec<&Expr> {
        self.main_boundary_constraints.last()
    }

    pub fn main_degrees(&self) -> Vec<u8> {
        self.main_transition_constraints.degrees()
    }

    pub fn main_transition_constraints(&self) -> &[NodeIndex] {
        self.main_transition_constraints.constraints()
    }

    pub fn main_transition_graph(&self) -> &AlgebraicGraph {
        self.main_transition_constraints.graph()
    }
}

// TODO: add checks for the correctness of the AirIR that is built.
#[cfg(test)]
mod tests {
    use super::*;
    use parser::{
        grammar::SourceParser,
        lexer::{Lexer, Token},
    };

    fn get_parsed(source: &str) -> ast::Source {
        let lex = Lexer::new(source).spanned().map(Token::to_spanned);
        SourceParser::new().parse(lex).unwrap()
    }

    #[test]
    fn boundary_constraints() {
        let source = "
        trace_columns:
            main: [clk]
        boundary_constraints:
            enf clk.first = 0
            enf clk.last = 1";

        let parsed = get_parsed(source);

        let result = AirIR::from_source(&parsed);
        assert!(result.is_ok());
    }

    #[test]
    fn err_bc_column_undeclared() {
        let source = "
        boundary_constraints:
            enf clk.first = 0
            enf clk.last = 1";

        let parsed = get_parsed(source);

        let result = AirIR::from_source(&parsed);
        assert!(result.is_err());
    }

    #[test]
    fn err_bc_duplicate_first() {
        let source = "
        trace_columns:
            main: [clk]
        boundary_constraints:
            enf clk.first = 0
            enf clk.first = 1";

        let parsed = get_parsed(source);
        let result = AirIR::from_source(&parsed);

        assert!(result.is_err());
    }

    #[test]
    fn err_bc_duplicate_last() {
        let source = "
        trace_columns:
            main: [clk]
        boundary_constraints:
            enf clk.last = 0
            enf clk.last = 1";

        let parsed = get_parsed(source);

        assert!(AirIR::from_source(&parsed).is_err());
    }

    #[test]
    fn transition_constraints() {
        let source = "
        trace_columns:
            main: [clk]
        transition_constraints:
            enf clk' = clk + 1";

        let parsed = get_parsed(source);

        let result = AirIR::from_source(&parsed);
        assert!(result.is_ok());
    }

    #[test]
    fn err_tc_column_undeclared() {
        let source = "
        transition_constraints:
            enf clk' = clk + 1";

        let parsed = get_parsed(source);

        let result = AirIR::from_source(&parsed);
        assert!(result.is_err());
    }
}
