use super::{build_parse_test, Identifier, Source};
use crate::ast::{
    ArgumentType, Expression::*, Function, FunctionArgument, FunctionBody, IntegrityConstraint,
    IntegrityStmt, ReturnType, SourceSection, Trace, TraceCols, Variable, VariableType,
    VectorAccess,
};
use air_script_core::{Expression, Iterable, ListComprehension};

#[test]
fn scalar_args_fn() {
    let source = "
        fn add_numbers(x: scalar, y: scalar) -> scalar:
            return x + y
        trace_columns:
            main: [a, b, c[10]]
        integrity_constraints:
            let k = add_numbers(a, b)
            enf c[3] = k";
    let expected = Source(vec![
        SourceSection::Function(Function::new(
            Identifier("add_numbers".to_string()),
            vec![
                FunctionArgument::new(Identifier("x".to_string()), ArgumentType::Scalar),
                FunctionArgument::new(Identifier("y".to_string()), ArgumentType::Scalar),
            ],
            vec![ReturnType::Scalar],
            FunctionBody::new(
                Vec::new(),
                vec![VariableType::Scalar(Expression::Add(
                    Box::new(Expression::Elem(Identifier("x".to_string()))),
                    Box::new(Expression::Elem(Identifier("y".to_string()))),
                ))],
            ),
        )),
        SourceSection::Trace(Trace {
            main_cols: vec![
                TraceCols::new(Identifier("a".to_string()), 1),
                TraceCols::new(Identifier("b".to_string()), 1),
                TraceCols::new(Identifier("c".to_string()), 10),
            ],
            aux_cols: vec![],
        }),
        SourceSection::IntegrityConstraints(vec![
            IntegrityStmt::Variable(Variable::new(
                Identifier("k".to_string()),
                VariableType::Scalar(FunctionCall(
                    Identifier("add_numbers".to_string()),
                    vec![
                        VariableType::Scalar(Expression::Elem(Identifier("a".to_string()))),
                        VariableType::Scalar(Expression::Elem(Identifier("b".to_string()))),
                    ],
                )),
            )),
            IntegrityStmt::Constraint(IntegrityConstraint::new(
                Expression::VectorAccess(VectorAccess::new(Identifier("c".to_string()), 3)),
                Expression::Elem(Identifier("k".to_string())),
            )),
        ]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn vector_arg_simple_fn() {
    let source = "
        fn double_array(x: vector[10]) -> vector[10]:
            return [x * 2 for x in x]
        trace_columns:
            main: [a, b, c[10]]
        integrity_constraints:
            let y = double_array(c)
            enf a = y[3]";
    let expected = Source(vec![
        SourceSection::Function(Function::new(
            Identifier("double_array".to_string()),
            vec![FunctionArgument::new(
                Identifier("x".to_string()),
                ArgumentType::Vector(10),
            )],
            vec![ReturnType::Vector(10)],
            FunctionBody::new(
                Vec::new(),
                vec![VariableType::ListComprehension(ListComprehension::new(
                    Expression::Mul(
                        Box::new(Expression::Elem(Identifier("x".to_string()))),
                        Box::new(Expression::Const(2)),
                    ),
                    vec![(
                        Identifier("x".to_string()),
                        Iterable::Identifier(Identifier("x".to_string())),
                    )],
                ))],
            ),
        )),
        SourceSection::Trace(Trace {
            main_cols: vec![
                TraceCols::new(Identifier("a".to_string()), 1),
                TraceCols::new(Identifier("b".to_string()), 1),
                TraceCols::new(Identifier("c".to_string()), 10),
            ],
            aux_cols: vec![],
        }),
        SourceSection::IntegrityConstraints(vec![
            IntegrityStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(FunctionCall(
                    Identifier("double_array".to_string()),
                    vec![VariableType::Scalar(Expression::Elem(Identifier(
                        "c".to_string(),
                    )))],
                )),
            )),
            IntegrityStmt::Constraint(IntegrityConstraint::new(
                Expression::Elem(Identifier("a".to_string())),
                Expression::VectorAccess(VectorAccess::new(Identifier("y".to_string()), 3)),
            )),
        ]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}

#[test]
fn two_vector_args_simple_fn() {
    let source = "
        fn vector_addition(a: vector[10], b: vector[10]) -> vector[10]:
            return [a + b for (a, b) in (a, b)]
        trace_columns:
            main: [a, b, c[10]]
        integrity_constraints:
            let y = double_array(c)
            enf a = y[3]";

    let expected = Source(vec![
        SourceSection::Function(Function::new(
            Identifier("vector_addition".to_string()),
            vec![
                FunctionArgument::new(Identifier("a".to_string()), ArgumentType::Vector(10)),
                FunctionArgument::new(Identifier("b".to_string()), ArgumentType::Vector(10)),
            ],
            vec![ReturnType::Vector(10)],
            FunctionBody::new(
                Vec::new(),
                vec![VariableType::ListComprehension(ListComprehension::new(
                    Expression::Add(
                        Box::new(Expression::Elem(Identifier("a".to_string()))),
                        Box::new(Expression::Elem(Identifier("b".to_string()))),
                    ),
                    vec![
                        (
                            Identifier("a".to_string()),
                            Iterable::Identifier(Identifier("a".to_string())),
                        ),
                        (
                            Identifier("b".to_string()),
                            Iterable::Identifier(Identifier("b".to_string())),
                        ),
                    ],
                ))],
            ),
        )),
        SourceSection::Trace(Trace {
            main_cols: vec![
                TraceCols::new(Identifier("a".to_string()), 1),
                TraceCols::new(Identifier("b".to_string()), 1),
                TraceCols::new(Identifier("c".to_string()), 10),
            ],
            aux_cols: vec![],
        }),
        SourceSection::IntegrityConstraints(vec![
            IntegrityStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(FunctionCall(
                    Identifier("double_array".to_string()),
                    vec![VariableType::Scalar(Expression::Elem(Identifier(
                        "c".to_string(),
                    )))],
                )),
            )),
            IntegrityStmt::Constraint(IntegrityConstraint::new(
                Expression::Elem(Identifier("a".to_string())),
                Expression::VectorAccess(VectorAccess::new(Identifier("y".to_string()), 3)),
            )),
        ]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}
