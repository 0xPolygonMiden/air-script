use super::SourceParser;
use crate::{
    ast::*,
    build_parse_test,
    error::{Error, ParseError},
};
use std::fs;

mod utils;

mod arithmetic_ops;
mod boundary_constraints;
mod comments;
mod constants;
mod identifiers;
mod integrity_constraints;
mod list_comprehension;
mod periodic_columns;
mod pub_inputs;
mod sections;
mod trace_columns;
mod variables;

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
        SourceSection::Trace(Trace {
            main_cols: vec![
                TraceCols::new(Identifier("clk".to_string()), 1),
                TraceCols::new(Identifier("fmp".to_string()), 1),
                TraceCols::new(Identifier("ctx".to_string()), 1),
            ],
            aux_cols: vec![],
        }),
        // integrity_constraints:
        //     enf clk' = clk + 1
        SourceSection::IntegrityConstraints(vec![IntegrityStmt::Constraint(
            IntegrityConstraint::new(
                // clk' = clk + 1
                Expression::NamedTraceAccess(NamedTraceAccess::new(
                    Identifier("clk".to_string()),
                    0,
                    1,
                )),
                Expression::Add(
                    Box::new(Expression::Elem(Identifier("clk".to_string()))),
                    Box::new(Expression::Const(1)),
                ),
            ),
        )]),
        // boundary_constraints:
        //     enf clk.first = 0
        SourceSection::BoundaryConstraints(vec![BoundaryStmt::Constraint(
            BoundaryConstraint::new(
                NamedTraceAccess::new(Identifier("clk".to_string()), 0, 0),
                Boundary::First,
                Expression::Const(0),
            ),
        )]),
    ]);
    build_parse_test!(source.as_str()).expect_ast(expected);
}
