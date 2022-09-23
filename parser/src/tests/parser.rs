use crate::{
    ast::{
        AirDef, Boundary, BoundaryConstraints, Constraint, Expr, Identifier, Source, SourceSection,
        TraceCols, TraceColsGrp, TraceColsGrpType, TransitionConstraints,
    },
    build_parse_test,
    error::{Error, ParseError::InvalidInt},
    lexer::Token,
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
        cols: vec![TraceColsGrp {
            cols_grp_type: TraceColsGrpType::MainTraceCols,
            cols: vec![
                Expr::Variable(Identifier {
                    name: "clk".to_string(),
                }),
                Expr::Variable(Identifier {
                    name: "fmp".to_string(),
                }),
                Expr::Variable(Identifier {
                    name: "ctx".to_string(),
                }),
            ],
        }],
    })]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn transition_constraints() {
    let source = "
    transition_constraints:
        enf clk' = clk + 1";
    let expected = Source(vec![SourceSection::TransitionConstraints(
        TransitionConstraints {
            transition_constraints: vec![Constraint {
                expr: Expr::Equal(
                    Box::new(Expr::Variable(Identifier {
                        name: "clk'".to_string(),
                    })),
                    Box::new(Expr::Add(
                        Box::new(Expr::Variable(Identifier {
                            name: "clk".to_string(),
                        })),
                        Box::new(Expr::Int(Token::Number("1".to_string()))),
                    )),
                ),
            }],
        },
    )]);
    build_parse_test!(source).expect_ast(expected);
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
                Constraint {
                    expr: Expr::Equal(
                        Box::new(Expr::Variable(Identifier {
                            name: "clk'".to_string(),
                        })),
                        Box::new(Expr::Add(
                            Box::new(Expr::Variable(Identifier {
                                name: "clk".to_string(),
                            })),
                            Box::new(Expr::Int(Token::Number("1".to_string()))),
                        )),
                    ),
                },
                Constraint {
                    expr: Expr::Equal(
                        Box::new(Expr::Subtract(
                            Box::new(Expr::Variable(Identifier {
                                name: "clk'".to_string(),
                            })),
                            Box::new(Expr::Variable(Identifier {
                                name: "clk".to_string(),
                            })),
                        )),
                        Box::new(Expr::Int(Token::Number("1".to_string()))),
                    ),
                },
            ],
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
            boundary_constraints: vec![Constraint {
                expr: Expr::Equal(
                    Box::new(Expr::Boundary(
                        Identifier {
                            name: "clk".to_string(),
                        },
                        Boundary::First,
                    )),
                    Box::new(Expr::Int(Token::Number("0".to_string()))),
                ),
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
                Constraint {
                    expr: Expr::Equal(
                        Box::new(Expr::Boundary(
                            Identifier {
                                name: "clk".to_string(),
                            },
                            Boundary::First,
                        )),
                        Box::new(Expr::Int(Token::Number("0".to_string()))),
                    ),
                },
                Constraint {
                    expr: Expr::Equal(
                        Box::new(Expr::Boundary(
                            Identifier {
                                name: "clk".to_string(),
                            },
                            Boundary::Last,
                        )),
                        Box::new(Expr::Int(Token::Number("1".to_string()))),
                    ),
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
            cols: vec![TraceColsGrp {
                cols_grp_type: TraceColsGrpType::MainTraceCols,
                // [clk, fmp, ctx]
                cols: vec![
                    Expr::Variable(Identifier {
                        name: "clk".to_string(),
                    }),
                    Expr::Variable(Identifier {
                        name: "fmp".to_string(),
                    }),
                    Expr::Variable(Identifier {
                        name: "ctx".to_string(),
                    }),
                ],
            }],
        }),
        // transition_constraints:
        //     enf clk' = clk + 1
        SourceSection::TransitionConstraints(TransitionConstraints {
            transition_constraints: vec![Constraint {
                // clk' = clk + 1
                expr: Expr::Equal(
                    Box::new(Expr::Variable(Identifier {
                        name: "clk'".to_string(),
                    })),
                    Box::new(Expr::Add(
                        Box::new(Expr::Variable(Identifier {
                            name: "clk".to_string(),
                        })),
                        Box::new(Expr::Int(Token::Number("1".to_string()))),
                    )),
                ),
            }],
        }),
        // boundary_constraints:
        //     enf clk.first = 0
        SourceSection::BoundaryConstraints(BoundaryConstraints {
            boundary_constraints: vec![Constraint {
                // enf clk.first = 0
                expr: Expr::Equal(
                    Box::new(Expr::Boundary(
                        Identifier {
                            name: "clk".to_string(),
                        },
                        Boundary::First,
                    )),
                    Box::new(Expr::Int(Token::Number("0".to_string()))),
                ),
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
        num.to_string()
    );
    // Integers can only be of type u64.
    let error = Error::ParseError(InvalidInt(format!("Int too big : {}", num.to_string())));
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
