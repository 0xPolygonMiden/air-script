use super::SourceParser;
use crate::{
    ast::*,
    build_parse_test,
    error::{Error, ParseError},
};
use std::fs;

mod utils;

mod boundary_constraints;
mod periodic_columns;
mod pub_inputs;
mod sections;
mod trace_columns;
mod transition_constraints;

mod comments;
mod constants;
mod expressions;
mod identifiers;

// FULL AIR FILE
// ================================================================================================

#[test]
fn full_air_file() {
    let source =
        fs::read_to_string("src/parser/tests/input/system.air").expect("Could not read file");
    let expected = Source(vec![
        // def SystemAir
        SourceSection::AirDef(Identifier("SystemAir".to_string())),
        // trace_columns:
        //     main: [clk, fmp, ctx]
        SourceSection::TraceCols(TraceCols {
            main_cols: vec![
                Identifier("clk".to_string()),
                Identifier("fmp".to_string()),
                Identifier("ctx".to_string()),
            ],
            aux_cols: vec![],
        }),
        // transition_constraints:
        //     enf clk' = clk + 1
        SourceSection::TransitionConstraints(TransitionConstraints {
            transition_constraints: vec![TransitionConstraint::new(
                // clk' = clk + 1
                TransitionExpr::Next(Identifier("clk".to_string())),
                TransitionExpr::Add(
                    Box::new(TransitionExpr::Elem(Identifier("clk".to_string()))),
                    Box::new(TransitionExpr::Const(1)),
                ),
            )],
        }),
        // boundary_constraints:
        //     enf clk.first = 0
        SourceSection::BoundaryConstraints(BoundaryConstraints {
            boundary_constraints: vec![BoundaryConstraint::new(
                Identifier("clk".to_string()),
                Boundary::First,
                BoundaryExpr::Const(0),
            )],
        }),
    ]);
    build_parse_test!(source.as_str()).expect_ast(expected);
}
