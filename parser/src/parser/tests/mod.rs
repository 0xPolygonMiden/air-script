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
mod evaluators;
mod identifiers;
mod integrity_constraints;
mod list_comprehension;
mod list_folding;
mod periodic_columns;
mod pub_inputs;
mod random_values;
mod sections;
mod selectors;
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
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("clk".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("fmp".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("ctx".to_string()), 0, 2, 1),
            TraceBinding::new(Identifier("$main".to_string()), 0, 0, 3),
        ]]),
        // integrity_constraints:
        //     enf clk' = clk + 1
        SourceSection::IntegrityConstraints(vec![IntegrityStmt::Constraint(
            ConstraintType::Inline(IntegrityConstraint::new(
                // clk' = clk + 1
                Expression::SymbolAccess(SymbolAccess::new(
                    Identifier("clk".to_string()),
                    AccessType::Default,
                    1,
                )),
                Expression::Add(
                    Box::new(Expression::SymbolAccess(SymbolAccess::new(
                        Identifier("clk".to_string()),
                        AccessType::Default,
                        0,
                    ))),
                    Box::new(Expression::Const(1)),
                ),
            )),
            None,
            None,
        )]),
        // boundary_constraints:
        //     enf clk.first = 0
        SourceSection::BoundaryConstraints(vec![BoundaryStmt::Constraint(
            BoundaryConstraint::new(
                SymbolAccess::new(Identifier("clk".to_string()), AccessType::Default, 0),
                Boundary::First,
                Expression::Const(0),
            ),
            None,
        )]),
    ]);
    build_parse_test!(source.as_str()).expect_ast(expected);
}
