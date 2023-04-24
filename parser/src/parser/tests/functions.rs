use super::{build_parse_test, Identifier, Source};
use crate::ast::{
    ArgumentType, ConstraintType, Expression::*, Function, FunctionParam, FunctionBody,
    IntegrityConstraint, IntegrityStmt, Iterable, ListComprehension, ReturnType, SourceSection,
    TraceBinding, Variable, VariableType, VectorAccess,
};

#[test]
fn scalar_args_fn() {
    let source = "
        fn add_numbers(x: scalar, y: scalar) -> scalar:
            return x + y
        trace_columns:
            main: [a, b, c[10]]
        integrity_constraints:
            let k = $add_numbers(a, b)
            enf c[3] = k";
    let expected = Source(vec![
        SourceSection::Function(Function::new(
            Identifier("add_numbers".to_string()),
            vec![
                FunctionParam::new(Identifier("x".to_string()), ArgumentType::Scalar),
                FunctionParam::new(Identifier("y".to_string()), ArgumentType::Scalar),
            ],
            vec![ReturnType::Scalar],
            FunctionBody::new(
                Vec::new(),
                vec![VariableType::Scalar(Add(
                    Box::new(Elem(Identifier("x".to_string()))),
                    Box::new(Elem(Identifier("y".to_string()))),
                ))],
            ),
        )),
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 10),
        ]]),
        SourceSection::IntegrityConstraints(vec![
            IntegrityStmt::Variable(Variable::new(
                Identifier("k".to_string()),
                VariableType::Scalar(FunctionCall(
                    Identifier("add_numbers".to_string()),
                    vec![
                        VariableType::Scalar(Elem(Identifier("a".to_string()))),
                        VariableType::Scalar(Elem(Identifier("b".to_string()))),
                    ],
                )),
            )),
            IntegrityStmt::Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    VectorAccess(VectorAccess::new(Identifier("c".to_string()), 3)),
                    Elem(Identifier("k".to_string())),
                )),
                None,
            ),
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
            let y = $double_array(c)
            enf a = y[3]";
    let expected = Source(vec![
        SourceSection::Function(Function::new(
            Identifier("double_array".to_string()),
            vec![FunctionParam::new(
                Identifier("x".to_string()),
                ArgumentType::Vector(10),
            )],
            vec![ReturnType::Vector(10)],
            FunctionBody::new(
                Vec::new(),
                vec![VariableType::ListComprehension(ListComprehension::new(
                    Mul(
                        Box::new(Elem(Identifier("x".to_string()))),
                        Box::new(Const(2)),
                    ),
                    vec![(
                        Identifier("x".to_string()),
                        Iterable::Identifier(Identifier("x".to_string())),
                    )],
                ))],
            ),
        )),
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 10),
        ]]),
        SourceSection::IntegrityConstraints(vec![
            IntegrityStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(FunctionCall(
                    Identifier("double_array".to_string()),
                    vec![VariableType::Scalar(Elem(Identifier("c".to_string())))],
                )),
            )),
            IntegrityStmt::Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    Elem(Identifier("a".to_string())),
                    VectorAccess(VectorAccess::new(Identifier("y".to_string()), 3)),
                )),
                None,
            ),
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
            let y = $double_array(c)
            enf a = y[3]";

    let expected = Source(vec![
        SourceSection::Function(Function::new(
            Identifier("vector_addition".to_string()),
            vec![
                FunctionParam::new(Identifier("a".to_string()), ArgumentType::Vector(10)),
                FunctionParam::new(Identifier("b".to_string()), ArgumentType::Vector(10)),
            ],
            vec![ReturnType::Vector(10)],
            FunctionBody::new(
                Vec::new(),
                vec![VariableType::ListComprehension(ListComprehension::new(
                    Add(
                        Box::new(Elem(Identifier("a".to_string()))),
                        Box::new(Elem(Identifier("b".to_string()))),
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
        SourceSection::Trace(vec![vec![
            TraceBinding::new(Identifier("a".to_string()), 0, 0, 1),
            TraceBinding::new(Identifier("b".to_string()), 0, 1, 1),
            TraceBinding::new(Identifier("c".to_string()), 0, 2, 10),
        ]]),
        SourceSection::IntegrityConstraints(vec![
            IntegrityStmt::Variable(Variable::new(
                Identifier("y".to_string()),
                VariableType::Scalar(FunctionCall(
                    Identifier("double_array".to_string()),
                    vec![VariableType::Scalar(Elem(Identifier("c".to_string())))],
                )),
            )),
            IntegrityStmt::Constraint(
                ConstraintType::Inline(IntegrityConstraint::new(
                    Elem(Identifier("a".to_string())),
                    VectorAccess(VectorAccess::new(Identifier("y".to_string()), 3)),
                )),
                None,
            ),
        ]),
    ]);
    build_parse_test!(source).expect_ast(expected);
}
