use crate::{
    ast::{
        AirDef, Boundary, BoundaryConstraint, BoundaryConstraints, Expr, Identifier, Source,
        SourceSection, TraceCols, TransitionConstraint, TransitionConstraints,
    },
    build_parse_test,
    error::{
        Error,
        ParseError::{InvalidInt, InvalidTraceCols},
    },
    tests::ParseTest,
};
use std::fs;

// SECTIONS
// ================================================================================================

#[test]
fn trace_columns() {
    let source = "
    trace_columns:
        main: [clk, fmp, ctx]";
    let expected = Source(vec![SourceSection::TraceCols(TraceCols {
        main_cols: vec![
            Identifier {
                name: "clk".to_string(),
            },
            Identifier {
                name: "fmp".to_string(),
            },
            Identifier {
                name: "ctx".to_string(),
            },
        ],
        aux_cols: vec![],
    })]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn trace_columns_main_and_aux() {
    let source = "
    trace_columns:
        main: [clk, fmp, ctx]
        aux: [rc_bus, ch_bus]";
    let expected = Source(vec![SourceSection::TraceCols(TraceCols {
        main_cols: vec![
            Identifier {
                name: "clk".to_string(),
            },
            Identifier {
                name: "fmp".to_string(),
            },
            Identifier {
                name: "ctx".to_string(),
            },
        ],
        aux_cols: vec![
            Identifier {
                name: "rc_bus".to_string(),
            },
            Identifier {
                name: "ch_bus".to_string(),
            },
        ],
    })]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn empty_trace_columns_error() {
    let source = "
    trace_columns:";
    // Trace columns cannot be empty
    let error = Error::ParseError(InvalidTraceCols(
        "Trace Columns cannot be empty".to_string(),
    ));
    build_parse_test!(source).expect_error(error);
}

#[test]
fn transition_constraints() {
    let source = "
    transition_constraints:
        enf clk' = clk + 1";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint {
                lhs: Expr::Next(Identifier {
                    name: "clk".to_string(),
                }),
                rhs: Expr::Add(
                    Box::new(Expr::Variable(Identifier {
                        name: "clk".to_string(),
                    })),
                    Box::new(Expr::Constant(1)),
                ),
            }],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraints_invalid() {
    let source = "transition_constraints:
        enf clk' = clk = 1";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn multiple_transition_constraints() {
    let source = "
    transition_constraints:
        enf clk' = clk + 1
        enf clk' - clk = 1";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![
                TransitionConstraint {
                    lhs: Expr::Next(Identifier {
                        name: "clk".to_string(),
                    }),
                    rhs: Expr::Add(
                        Box::new(Expr::Variable(Identifier {
                            name: "clk".to_string(),
                        })),
                        Box::new(Expr::Constant(1)),
                    ),
                },
                TransitionConstraint {
                    lhs: Expr::Subtract(
                        Box::new(Expr::Next(Identifier {
                            name: "clk".to_string(),
                        })),
                        Box::new(Expr::Variable(Identifier {
                            name: "clk".to_string(),
                        })),
                    ),
                    rhs: Expr::Constant(1),
                },
            ],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multi_arithmetic_ops() {
    let source = "
    transition_constraints:
        enf clk' - clk - 1 = 0";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![TransitionConstraint {
                lhs: Expr::Subtract(
                    Box::new(Expr::Subtract(
                        Box::new(Expr::Next(Identifier {
                            name: "clk".to_string(),
                        })),
                        Box::new(Expr::Variable(Identifier {
                            name: "clk".to_string(),
                        })),
                    )),
                    Box::new(Expr::Constant(1)),
                ),
                rhs: Expr::Constant(0),
            }],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn boundary_constraints() {
    let source = "
    boundary_constraints:
        enf clk.first = 0";
    let expected = Source(vec![SourceSection::BoundaryConstraints(
        BoundaryConstraints {
            boundary_constraints: vec![BoundaryConstraint {
                column: Identifier {
                    name: "clk".to_string(),
                },
                boundary: Boundary::First,
                value: Expr::Constant(0),
            }],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn multiple_boundary_constraints() {
    let source = "
    boundary_constraints:
        enf clk.first = 0
        enf clk.last = 1";
    let expected = Source(vec![SourceSection::BoundaryConstraints(
        BoundaryConstraints {
            boundary_constraints: vec![
                BoundaryConstraint {
                    column: Identifier {
                        name: "clk".to_string(),
                    },
                    boundary: Boundary::First,
                    value: Expr::Constant(0),
                },
                BoundaryConstraint {
                    column: Identifier {
                        name: "clk".to_string(),
                    },
                    boundary: Boundary::Last,
                    value: Expr::Constant(1),
                },
            ],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
}

// AIR FILE
// ================================================================================================

#[test]
fn full_air_file() {
    let source = fs::read_to_string("src/tests/input/system.air").expect("Could not read file");
    let expected = Source(vec![
        // def SystemAir
        SourceSection::AirDef(AirDef {
            name: Identifier {
                name: "SystemAir".to_string(),
            },
        }),
        // trace_columns:
        //     main: [clk, fmp, ctx]
        SourceSection::TraceCols(TraceCols {
            main_cols: vec![
                Identifier {
                    name: "clk".to_string(),
                },
                Identifier {
                    name: "fmp".to_string(),
                },
                Identifier {
                    name: "ctx".to_string(),
                },
            ],
            aux_cols: vec![],
        }),
        // transition_constraints:
        //     enf clk' = clk + 1
        SourceSection::TransitionConstraints(TransitionConstraints {
            transition_constraints: vec![TransitionConstraint {
                // clk' = clk + 1
                lhs: Expr::Next(Identifier {
                    name: "clk".to_string(),
                }),
                rhs: Expr::Add(
                    Box::new(Expr::Variable(Identifier {
                        name: "clk".to_string(),
                    })),
                    Box::new(Expr::Constant(1)),
                ),
            }],
        }),
        // boundary_constraints:
        //     enf clk.first = 0
        SourceSection::BoundaryConstraints(BoundaryConstraints {
            boundary_constraints: vec![BoundaryConstraint {
                column: Identifier {
                    name: "clk".to_string(),
                },
                boundary: Boundary::First,
                value: Expr::Constant(0),
            }],
        }),
    ]);
    build_parse_test!(source.as_str()).expect_ast(expected);
}

// PARSE ERRORS
// ================================================================================================

#[test]
fn error_invalid_int() {
    let num: u128 = u64::max_value() as u128 + 1;
    let source = format!(
        "
    transition_constraints:
        enf clk' = clk + {}",
        num
    );
    // Integers can only be of type u64.
    let error = Error::ParseError(InvalidInt(format!("Int too big : {}", num)));
    build_parse_test!(source.as_str()).expect_error(error);
}

// UNRECOGNIZED TOKEN ERRORS
// ================================================================================================

#[test]
fn error_constraint_without_section() {
    // Constraints outside of valid sections are not allowed.
    let source = "enf clk' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn error_identifier_starting_with_int() {
    // Identifiers cannot start with numeric characters.
    // lexer considers the integer 1 and alphabetic clk' to be separate tokens
    // hence this fails at parser level since a valid identifier is expected
    // at that position which 1 is not.
    let source = "
    transition_constraints:
        enf 1clk' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();
}

#[test]
fn error_invalid_next_usage() {
    let source = "
    transition_constraints:
        enf clk'' = clk + 1";
    build_parse_test!(source).expect_unrecognized_token();
}
