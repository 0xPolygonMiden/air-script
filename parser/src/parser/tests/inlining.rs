use air_pass::Pass;
use miden_diagnostics::SourceSpan;

use pretty_assertions::assert_eq;

use crate::{
    ast::*,
    transforms::{ConstantPropagation, Inlining},
};

use super::ParseTest;

/// This test inlines an evaluator function into the root
/// integrity constraints. The evaluator is called with a
/// single trace column binding representing two columns
/// in the main trace, which is split into two individual
/// bindings via the evaluator function signature.
///
/// It is expected that the resulting evaluator function
/// body will have its references to those parameters rewritten
/// to refer to the input binding, but with appropriate accesses
/// inserted to match the semantics of the function signature
#[test]
fn test_inlining_with_evaluator_split_input_binding() {
    let root = r#"
    def root

    use lib::*

    trace_columns {
        main: [clk, a, b[2], c]
    }

    public_inputs {
        inputs: [0]
    }

    const A = [2, 4, 6, 8]
    const B = [[1, 1], [2, 2]]

    integrity_constraints {
        enf test_constraint(b)
        let x = 2^EXP
        let y = A[0..2]
        enf a + y[1] = c + (x + 1)
    }

    boundary_constraints {
        let x = B[0]
        enf a.first = x[0]
    }

    "#;
    let lib = r#"
    mod lib

    const EXP = 2

    ev test_constraint([b0, b1]) {
        let x = EXP
        let y = 2^x
        enf b0 + x = b1 + y
    }"#;

    let test = ParseTest::new();
    let path = std::env::current_dir().unwrap().join("lib.air");
    test.add_virtual_file(path, lib.to_string());

    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (a, 1), (b, 2), (c, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 0),
    );
    expected
        .constants
        .insert(ident!(root, A), constant!(A = [2, 4, 6, 8]));
    expected
        .constants
        .insert(ident!(root, B), constant!(B = [[1, 1], [2, 2]]));
    expected
        .constants
        .insert(ident!(lib, EXP), constant!(EXP = 2));
    // When constant propagation and inlining is done, the boundary constraints should look like:
    //     enf a.first = 1
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(a, Boundary::First, Type::Felt),
        int!(1)
    )));
    // When constant propagation and inlining is done, the integrity constraints should look like:
    //     enf b[0] + 2 = b[1] + 4
    //     enf a + 4 = c + 5
    expected.integrity_constraints.push(enforce!(eq!(
        add!(access!(b[0], Type::Felt), int!(2)),
        add!(access!(b[1], Type::Felt), int!(4))
    )));
    expected.integrity_constraints.push(enforce!(eq!(
        add!(access!(a, Type::Felt), int!(4)),
        add!(access!(c, Type::Felt), int!(5))
    )));
    // The test_constraint function before inlining should look like:
    //     enf b0 + 2 = b1 + 4
    let body = vec![enforce!(eq!(
        add!(access!(b0, Type::Felt), int!(2)),
        add!(access!(b1, Type::Felt), int!(4))
    ))];
    expected.evaluators.insert(
        function_ident!(lib, test_constraint),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(test_constraint),
            vec![trace_segment!(0, "%0", [(b0, 1), (b1, 1)])],
            body,
        ),
    );

    assert_eq!(program, expected);
}

/// This test inlines an evaluator function into the root
/// integrity constraints. The evaluator is called with two
/// disjoint bindings representing three columns from the main
/// trace, packed using a vector literal. The evaluator function
/// then destructures that vector into a set of two bindings which
/// recombines the input columns into different groupings, and then
/// expresses a constraint using accesses into those groups.
///
/// It is expected that the resulting evaluator function
/// body will have its references to those parameters rewritten
/// to accesses relative to the input bindings, or to direct accesses
/// to the corresponding trace segment declaration.
#[test]
fn test_inlining_with_vector_literal_binding_regrouped() {
    let root = r#"
    def root

    use lib::*

    trace_columns {
        main: [clk, a, b[2], c]
    }

    public_inputs {
        inputs: [0]
    }

    integrity_constraints {
        enf test_constraint([clk, b])
    }

    boundary_constraints {
        enf clk.first = 0
    }

    "#;
    let lib = r#"
    mod lib

    ev test_constraint([pair[2], b1]) {
        enf pair[0] + pair[1] = b1
    }"#;

    let test = ParseTest::new();
    let path = std::env::current_dir().unwrap().join("lib.air");
    test.add_virtual_file(path, lib.to_string());

    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (a, 1), (b, 2), (c, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 0),
    );
    // The sole boundary constraint is already minimal
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(clk, Boundary::First, Type::Felt),
        int!(0)
    )));
    // When constant propagation and inlining is done, integrity_constraints should look like:
    //     enf clk + b[0] = b[1]
    expected.integrity_constraints.push(enforce!(eq!(
        add!(access!(clk, Type::Felt), access!(b[0], Type::Felt)),
        access!(b[1], Type::Felt)
    )));
    // The test_constraint function before inlining should look like:
    //     enf pair[0] + pair[1] = b1
    let body = vec![enforce!(eq!(
        add!(access!(pair[0], Type::Felt), access!(pair[1], Type::Felt)),
        access!(b1, Type::Felt)
    ))];
    expected.evaluators.insert(
        function_ident!(lib, test_constraint),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(test_constraint),
            vec![trace_segment!(0, "%0", [(pair, 2), (b1, 1)])],
            body,
        ),
    );

    assert_eq!(program, expected);
}

/// This test checks that there are no assumptions about the ordering of
/// arguments to an evaluator, i.e. there is no assumption that two consecutive
/// columns necessarily appear in that order in the trace_columns declaration
#[test]
fn test_inlining_with_vector_literal_binding_unordered() {
    let root = r#"
    def root

    use lib::*

    trace_columns {
        main: [clk, a, b[2], c]
    }

    public_inputs {
        inputs: [0]
    }

    integrity_constraints {
        enf test_constraint([b, clk])
    }

    boundary_constraints {
        enf clk.first = 0
    }
    "#;
    let lib = r#"
    mod lib

    ev test_constraint([b0, pair[2]]) {
        enf pair[1] + b0 = pair[0]
    }"#;

    let test = ParseTest::new();
    let path = std::env::current_dir().unwrap().join("lib.air");
    test.add_virtual_file(path, lib.to_string());

    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (a, 1), (b, 2), (c, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 0),
    );
    // The sole boundary constraint is already minimal
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(clk, Boundary::First, Type::Felt),
        int!(0)
    )));
    // When constant propagation and inlining is done, integrity_constraints should look like:
    //     enf clk + b[0] = b[1]
    expected.integrity_constraints.push(enforce!(eq!(
        add!(access!(clk, Type::Felt), access!(b[0], Type::Felt)),
        access!(b[1], Type::Felt)
    )));
    // The test_constraint function before inlining should look like:
    //     enf pair[1] + b0 = pair[0]
    let body = vec![enforce!(eq!(
        add!(access!(pair[1], Type::Felt), access!(b0, Type::Felt)),
        access!(pair[0], Type::Felt)
    ))];
    expected.evaluators.insert(
        function_ident!(lib, test_constraint),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(test_constraint),
            vec![trace_segment!(0, "%0", [(b0, 1), (pair, 2)])],
            body,
        ),
    );

    assert_eq!(program, expected);
}

/// This test checks the behavior when there are not only disjoint args/params
/// in a call to an evaluator, but that the number of arguments and parameters
/// is different, with more input arguments than parameter bindings.
#[test]
fn test_inlining_with_vector_literal_binding_different_arity_many_to_few() {
    let root = r#"
    def root

    use lib::*

    trace_columns {
        main: [clk, a, b[2], c]
    }

    public_inputs {
        inputs: [0]
    }

    integrity_constraints {
        enf test_constraint([clk, b, a])
    }

    boundary_constraints {
        enf clk.first = 0
    }
    "#;
    let lib = r#"
    mod lib

    ev test_constraint([pair[3], foo]) {
        enf pair[0] + pair[1] = foo + pair[2]
    }"#;

    let test = ParseTest::new();
    let path = std::env::current_dir().unwrap().join("lib.air");
    test.add_virtual_file(path, lib.to_string());

    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (a, 1), (b, 2), (c, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 0),
    );
    // The sole boundary constraint is already minimal
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(clk, Boundary::First, Type::Felt),
        int!(0)
    )));
    // When constant propagation and inlining is done, integrity_constraints should look like:
    //     enf clk + b[0] = a + b[1]
    expected.integrity_constraints.push(enforce!(eq!(
        add!(access!(clk, Type::Felt), access!(b[0], Type::Felt)),
        add!(access!(a, Type::Felt), access!(b[1], Type::Felt))
    )));
    // The test_constraint function before inlining should look like:
    //     enf pair[0] + pair[1] = a + pair[2]
    let body = vec![enforce!(eq!(
        add!(access!(pair[0], Type::Felt), access!(pair[1], Type::Felt)),
        add!(access!(foo, Type::Felt), access!(pair[2], Type::Felt))
    ))];
    expected.evaluators.insert(
        function_ident!(lib, test_constraint),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(test_constraint),
            vec![trace_segment!(0, "%0", [(pair, 3), (foo, 1)])],
            body,
        ),
    );

    assert_eq!(program, expected);
}

/// This test checks the behavior when there are not only disjoint args/params
/// in a call to an evaluator, but that the number of arguments and parameters
/// is different, with more parameter bindings than input arguments.
#[test]
fn test_inlining_with_vector_literal_binding_different_arity_few_to_many() {
    let root = r#"
    def root

    use lib::*

    trace_columns {
        main: [clk, a, b[2], c]
    }

    public_inputs {
        inputs: [0]
    }

    integrity_constraints {
        enf test_constraint([b, a])
    }

    boundary_constraints {
        enf clk.first = 0
    }
    "#;
    let lib = r#"
    mod lib

    ev test_constraint([x, y, z]) {
        enf x + y = z
    }"#;

    let test = ParseTest::new();
    let path = std::env::current_dir().unwrap().join("lib.air");
    test.add_virtual_file(path, lib.to_string());

    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (a, 1), (b, 2), (c, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 0),
    );
    // The sole boundary constraint is already minimal
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(clk, Boundary::First, Type::Felt),
        int!(0)
    )));
    // When constant propagation and inlining is done, integrity_constraints should look like:
    //     enf b[0] + b[1] = a
    expected.integrity_constraints.push(enforce!(eq!(
        add!(access!(b[0], Type::Felt), access!(b[1], Type::Felt)),
        access!(a, Type::Felt)
    )));
    // The test_constraint function before inlining should look like:
    //     enf x + y = z
    let body = vec![enforce!(eq!(
        add!(access!(x, Type::Felt), access!(y, Type::Felt)),
        access!(z, Type::Felt)
    ))];
    expected.evaluators.insert(
        function_ident!(lib, test_constraint),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(test_constraint),
            vec![trace_segment!(0, "%0", [(x, 1), (y, 1), (z, 1)])],
            body,
        ),
    );

    assert_eq!(program, expected);
}

/// This test checks the behavior when inlining across multiple modules with
/// nested calls to evaluators, with a mix of parameter/argument binding configurations
#[test]
fn test_inlining_across_modules_with_nested_evaluators_variant1() {
    let root = r#"
    def root

    use lib1::test_constraint

    trace_columns {
        main: [clk, a, b[2], c]
    }

    public_inputs {
        inputs: [0]
    }

    integrity_constraints {
        enf test_constraint([clk, b, a])
    }

    boundary_constraints {
        enf clk.first = 0
    }
    "#;
    let lib1 = r#"
    mod lib1

    use lib2::*

    ev test_constraint([tuple[3], z]) {
        enf helper_constraint([z, tuple[1..3]])
    }"#;
    let lib2 = r#"
    mod lib2

    ev helper_constraint([x[2], y]) {
        enf x[0] + x[1] = y
    }"#;

    let test = ParseTest::new();
    let path = std::env::current_dir().unwrap().join("lib1.air");
    test.add_virtual_file(path, lib1.to_string());
    let path = std::env::current_dir().unwrap().join("lib2.air");
    test.add_virtual_file(path, lib2.to_string());

    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (a, 1), (b, 2), (c, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 0),
    );
    // The sole boundary constraint is already minimal
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(clk, Boundary::First, Type::Felt),
        int!(0)
    )));
    // When constant propagation and inlining is done, integrity_constraints should look like:
    //     enf a + b[0] = b[1]
    expected.integrity_constraints.push(enforce!(eq!(
        add!(access!(a, Type::Felt), access!(b[0], Type::Felt)),
        access!(b[1], Type::Felt)
    )));
    // The test_constraint function before inlining should look like:
    //     enf helper_constraint([z, tuple[1..3]])
    let body = vec![enforce!(call!(lib2::helper_constraint(vector!(
        access!(z, Type::Felt),
        slice!(tuple, 1..3, Type::Vector(2))
    ))))];
    expected.evaluators.insert(
        function_ident!(lib1, test_constraint),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(test_constraint),
            vec![trace_segment!(0, "%0", [(tuple, 3), (z, 1)])],
            body,
        ),
    );
    // The helper_constraint function before inlining should look like:
    //     enf x[0] + x[1] = y
    let body = vec![enforce!(eq!(
        add!(access!(x[0], Type::Felt), access!(x[1], Type::Felt)),
        access!(y, Type::Felt)
    ))];
    expected.evaluators.insert(
        function_ident!(lib2, helper_constraint),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(helper_constraint),
            vec![trace_segment!(0, "%0", [(x, 2), (y, 1)])],
            body,
        ),
    );

    assert_eq!(program, expected);
}

/// This test is like *_variant1, but with a different mix of parameter/argument configurations
#[test]
fn test_inlining_across_modules_with_nested_evaluators_variant2() {
    let root = r#"
    def root

    use lib1::test_constraint

    trace_columns {
        main: [clk, a, b[2], c]
    }

    public_inputs {
        inputs: [0]
    }

    integrity_constraints {
        enf test_constraint([clk, b[0..2], a])
    }

    boundary_constraints {
        enf clk.first = 0
    }
    "#;
    let lib1 = r#"
    mod lib1

    use lib2::*

    ev test_constraint([tuple[3], z]) {
        enf helper_constraint([z, tuple[1], tuple[2..3]])
    }"#;
    let lib2 = r#"
    mod lib2

    ev helper_constraint([x[2], y]) {
        enf x[0] + x[1] = y
    }"#;

    let test = ParseTest::new();
    let path = std::env::current_dir().unwrap().join("lib1.air");
    test.add_virtual_file(path, lib1.to_string());
    let path = std::env::current_dir().unwrap().join("lib2.air");
    test.add_virtual_file(path, lib2.to_string());

    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (a, 1), (b, 2), (c, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 0),
    );
    // The sole boundary constraint is already minimal
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(clk, Boundary::First, Type::Felt),
        int!(0)
    )));
    // When constant propagation and inlining is done, integrity_constraints should look like:
    //     enf a + b[0] = b[1]
    expected.integrity_constraints.push(enforce!(eq!(
        add!(access!(a, Type::Felt), access!(b[0], Type::Felt)),
        access!(b[1], Type::Felt)
    )));
    // The test_constraint function before inlining should look like:
    //     enf helper_constraint([z, tuple[1..3]])
    let body = vec![enforce!(call!(lib2::helper_constraint(vector!(
        access!(z, Type::Felt),
        access!(tuple[1], Type::Felt),
        slice!(tuple, 2..3, Type::Vector(1))
    ))))];
    expected.evaluators.insert(
        function_ident!(lib1, test_constraint),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(test_constraint),
            vec![trace_segment!(0, "%0", [(tuple, 3), (z, 1)])],
            body,
        ),
    );
    // The helper_constraint function before inlining should look like:
    //     enf x[0] + x[1] = y
    let body = vec![enforce!(eq!(
        add!(access!(x[0], Type::Felt), access!(x[1], Type::Felt)),
        access!(y, Type::Felt)
    ))];
    expected.evaluators.insert(
        function_ident!(lib2, helper_constraint),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(helper_constraint),
            vec![trace_segment!(0, "%0", [(x, 2), (y, 1)])],
            body,
        ),
    );

    assert_eq!(program, expected);
}

/// This test verifies that constraint comprehensions (without a selector) are unrolled properly during inlining
///
/// In this variant, we do not involve other modules to keep the test focused on just the
/// comprehension unrolling behavior. Other tests will expand on this to test it when combined
/// with other inlining behavior.
#[test]
fn test_inlining_constraint_comprehensions_no_selector() {
    let root = r#"
    def root

    const YS = [2, 4, 6, 8]

    trace_columns {
        main: [clk, a, b[2], c]
    }

    public_inputs {
        inputs: [0]
    }

    integrity_constraints {
        # We're expecting this to expand to:
        #
        #    enf b[0]' = 2
        #    enf b[1]' = 4
        #
        enf x' = y for (x, y) in (b, YS[0..2])
    }

    boundary_constraints {
        enf clk.first = 0
    }
    "#;

    let test = ParseTest::new();
    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected
        .constants
        .insert(ident!(root, YS), constant!(YS = [2, 4, 6, 8]));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (a, 1), (b, 2), (c, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 0),
    );
    // The sole boundary constraint is already minimal
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(clk, Boundary::First, Type::Felt),
        int!(0)
    )));
    // When constant propagation and inlining is done, integrity_constraints should look like:
    //     enf b[0]' = 2
    //     enf b[1]' = 4
    expected
        .integrity_constraints
        .push(enforce!(eq!(access!(b[0], 1, Type::Felt), int!(2))));
    expected
        .integrity_constraints
        .push(enforce!(eq!(access!(b[1], 1, Type::Felt), int!(4))));

    assert_eq!(program, expected);
}

/// This test verifies that constraint comprehensions (with a selector) are unrolled properly during inlining
///
/// In this variant, we do not involve other modules to keep the test focused on just the
/// comprehension unrolling behavior. Other tests will expand on this to test it when combined
/// with other inlining behavior.
#[test]
fn test_inlining_constraint_comprehensions_with_selector() {
    let root = r#"
    def root

    const YS = [2, 4, 6, 8]

    trace_columns {
        main: [clk, a, b[2], c]
    }

    public_inputs {
        inputs: [0]
    }

    integrity_constraints {
        # We're expecting this to expand to:
        #
        #    enf b[0]' = 2 when c
        #    enf b[1]' = 4 when c
        #
        enf x' = y for (x, y) in (b, YS[0..2]) when c
    }

    boundary_constraints {
        enf clk.first = 0
    }
    "#;

    let test = ParseTest::new();
    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected
        .constants
        .insert(ident!(root, YS), constant!(YS = [2, 4, 6, 8]));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (a, 1), (b, 2), (c, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 0),
    );
    // The sole boundary constraint is already minimal
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(clk, Boundary::First, Type::Felt),
        int!(0)
    )));
    // When constant propagation and inlining is done, integrity_constraints should look like:
    //     enf b[0]' = 2 when c
    //     enf b[1]' = 4 when c
    expected
        .integrity_constraints
        .push(enforce!(eq!(access!(b[0], 1, Type::Felt), int!(2)), when access!(c, Type::Felt)));
    expected
        .integrity_constraints
        .push(enforce!(eq!(access!(b[1], 1, Type::Felt), int!(4)), when access!(c, Type::Felt)));

    assert_eq!(program, expected);
}

/// This test verifies that constraint comprehensions (with a selector) are unrolled properly during inlining.
/// Specifically, in this case we expect that because the selector is constant, only constraints for which the
/// selector is "truthy" (i.e. non-zero) remain, and that the selector has been elided.
///
/// In this variant, we do not involve other modules to keep the test focused on just the
/// comprehension unrolling behavior. Other tests will expand on this to test it when combined
/// with other inlining behavior.
#[test]
fn test_inlining_constraint_comprehensions_with_constant_selector() {
    let root = r#"
    def root

    const YS = [0, 4, 0, 8]

    trace_columns {
        main: [clk, a, b[4], c]
    }

    public_inputs {
        inputs: [0]
    }

    integrity_constraints {
        # We're expecting this to expand to:
        #
        #    enf b[1]' = 4
        #    enf b[3]' = 8
        #
        enf x' = y for (x, y) in (b, YS) when y
    }

    boundary_constraints {
        enf clk.first = 0
    }
    "#;

    let test = ParseTest::new();
    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected
        .constants
        .insert(ident!(root, YS), constant!(YS = [0, 4, 0, 8]));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (a, 1), (b, 4), (c, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 0),
    );
    // The sole boundary constraint is already minimal
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(clk, Boundary::First, Type::Felt),
        int!(0)
    )));
    // When constant propagation and inlining is done, integrity_constraints should look like:
    //     enf b[1]' = 4
    //     enf b[3]' = 8
    expected
        .integrity_constraints
        .push(enforce!(eq!(access!(b[1], 1, Type::Felt), int!(4))));
    expected
        .integrity_constraints
        .push(enforce!(eq!(access!(b[3], 1, Type::Felt), int!(8))));

    assert_eq!(program, expected);
}

/// This test verifies that constraint comprehensions present in evaluators are inlined into call sites correctly
///
/// This test exercises multiple edge cases in constant propagation/inlining in conjunction to make sure that all
/// of the pieces integrate together even in odd scenarios
#[test]
fn test_inlining_constraint_comprehensions_in_evaluator() {
    let root = r#"
    def root

    const YS = [0, 4, 0, 8]

    trace_columns {
        main: [clk, a, b[4], c]
    }

    public_inputs {
        inputs: [0]
    }

    integrity_constraints {
        enf test_constraint(b[1..4])
    }

    boundary_constraints {
        enf clk.first = 0
    }

    ev test_constraint([i, j[2]]) {
        let ys = [x^2 for x in YS]
        let k = j[0]
        let l = j[1]
        let xs = [i, k, l]
        enf x' = y for (x, y) in (xs, ys[1..4]) when y
    }"#;

    let test = ParseTest::new();
    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected
        .constants
        .insert(ident!(root, YS), constant!(YS = [0, 4, 0, 8]));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (a, 1), (b, 4), (c, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 0),
    );
    // The sole boundary constraint is already minimal
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(clk, Boundary::First, Type::Felt),
        int!(0)
    )));
    // When constant propagation and inlining is done, integrity_constraints should look like:
    //     enf b[1]' = 16
    //     enf b[3]' = 64
    expected
        .integrity_constraints
        .push(enforce!(eq!(access!(b[1], 1, Type::Felt), int!(16))));
    expected
        .integrity_constraints
        .push(enforce!(eq!(access!(b[3], 1, Type::Felt), int!(64))));
    // The evaluator definition is never modified by inlining, but is by constant propagation:
    //
    // ev test_constraint([i, j[2]]) {
    //     let k = j[0]
    //     let l = j[1]
    //     let xs = [i, k, l]
    //     enf x' = y for (x, y) in (xs, [16, 0, 64]) when y
    // }
    let body = vec![let_!(k = expr!(access!(j[0], Type::Felt))
        => let_!(l = expr!(access!(j[1], Type::Felt))
            => let_!(xs = vector!(access!(i, Type::Felt), access!(k, Type::Felt), access!(l, Type::Felt))
                => enforce_all!(lc!(((x, expr!(access!(xs, Type::Vector(3)))), (y, vector!(16, 0, 64)))
                    => eq!(access!(x, 1, Type::Felt), access!(y, Type::Felt)), when access!(y, Type::Felt)))))
    )];
    expected.evaluators.insert(
        function_ident!(root, test_constraint),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(test_constraint),
            vec![trace_segment!(0, "%0", [(i, 1), (j, 2)])],
            body,
        ),
    );

    assert_eq!(program, expected);
}

/// This test verifies that constraints involving let-bound, folded comprehensions behave as expected
#[test]
fn test_inlining_constraints_with_folded_comprehensions_in_evaluator() {
    let root = r#"
    def root

    trace_columns {
        main: [clk, a, b[4], c]
    }

    public_inputs {
        inputs: [0]
    }

    integrity_constraints {
        enf test_constraint(b[1..4])
    }

    boundary_constraints {
        enf clk.first = 0
    }

    ev test_constraint([x, ys[2]]) {
        let y = sum([col^7 for col in ys])
        let z = prod([col^7 for col in ys])
        enf x = y + z
    }"#;

    let test = ParseTest::new();
    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [(clk, 1), (a, 1), (b, 4), (c, 1)]
    ));
    expected.public_inputs.insert(
        ident!(inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(inputs), 0),
    );
    // The sole boundary constraint is already minimal
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(clk, Boundary::First, Type::Felt),
        int!(0)
    )));
    // When constant propagation and inlining is done, integrity_constraints should look like:
    //     let lc%0 = b[2]^7
    //     let lc%1 = b[3]^7
    //     let y = lc%0 + lc%1
    //     let lc%2 = b[2]^7
    //     let lc%3 = b[3]^7
    //     let z = lc%2 + lc%3
    //     enf b[1] = y + z
    expected
        .integrity_constraints
        .push(let_!("%lc0" = expr!(exp!(access!(b[2], Type::Felt), int!(7)))
            => let_!("%lc1" = expr!(exp!(access!(b[3], Type::Felt), int!(7)))
            => let_!(y = expr!(add!(access!("%lc0", Type::Felt), access!("%lc1", Type::Felt)))
            => let_!("%lc2" = expr!(exp!(access!(b[2], Type::Felt), int!(7)))
            => let_!("%lc3" = expr!(exp!(access!(b[3], Type::Felt), int!(7)))
            => let_!(z = expr!(mul!(access!("%lc2", Type::Felt), access!("%lc3", Type::Felt)))
            => enforce!(eq!(access!(b[1], Type::Felt), add!(access!(y, Type::Felt), access!(z, Type::Felt)))))))))));
    // The evaluator definition is never modified by constant propagation or inlining
    let body = vec![
        let_!(y = expr!(call!(sum(expr!(lc!(((col, expr!(access!(ys, Type::Vector(2))))) => exp!(access!(col, Type::Felt), int!(7)))))))
            => let_!(z = expr!(call!(prod(expr!(lc!(((col, expr!(access!(ys, Type::Vector(2))))) => exp!(access!(col, Type::Felt), int!(7)))))))
                => enforce!(eq!(access!(x, Type::Felt), add!(access!(y, Type::Felt), access!(z, Type::Felt)))))),
    ];
    expected.evaluators.insert(
        function_ident!(root, test_constraint),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(test_constraint),
            vec![trace_segment!(0, "%0", [(x, 1), (ys, 2)])],
            body,
        ),
    );

    assert_eq!(program, expected);
}

/// This test originally reproduced the bug in air-script#340, but as of this commit
/// that bug is fixed. This test remains not to prevent regressions necessarily, but
/// to add a more realistic test case to our test suite, and potentially catch bugs
/// we accidentally introduce in the future which blow up on this code.
#[test]
fn test_repro_issue340() {
    // NOTE: This code is exactly what was written in #340, but the only significant
    // part is the fact that a panic was raised due to visiting one of the folded
    // compehensions in `imm_reconstruction` as if it was a constraint comprehension,
    // due to incorrectly propagating the constraint flag in that situation. The
    // bug no longer exists, but this test program may be useful for other tests of
    // a more real-world nature, so it is preserved here
    let root = r#"
    def root

    trace_columns {
        main: [instruction_word, instruction_bits[32], immediate, s]
    }

    public_inputs {
        stack_inputs: [16]
        stack_outputs: [16]
    }

    periodic_columns {
        k0: [1, 1, 1, 1, 1, 1, 1, 0]
    }

    boundary_constraints {
        # define boundary constraints against the main trace at the first row of the trace.
        enf instruction_word.first = 0
        enf instruction_word.last = 2
    }

    integrity_constraints {
        # The instruction bit decomposition must be bits
        enf b^2 = b for b in instruction_bits

        # Ensure they add up to the instruction word:
        let word_sum = sum([2^i * a for (i, a) in (0..32, instruction_bits)])
        enf instruction_word = word_sum

        enf match {
            case s: imm_reconstruction([instruction_bits, immediate])
            case !s: immediate = 0
        }
    }

    # The highest bit is a sign bit, so we sign extend then reconstruct from the other bits
    ev imm_reconstruction([instruction_bits[32], immediate]) {
        let sign_bit = instruction_bits[31]
        let high_bit_sum = sum([sign_bit*2^i for i in (11..32)])
        let immediate_bits = instruction_bits[20..31]
        let low_bit_sum = sum([immediate_bit * 2^i for (i, immediate_bit) in (0..11, instruction_bits[20..31])])
        enf immediate = low_bit_sum + high_bit_sum
        enf sign_bit = 1
    }"#;

    let test = ParseTest::new();
    let program = match test.parse_program(root) {
        Err(err) => {
            test.diagnostics.emit(err);
            panic!("expected parsing to succeed, see diagnostics for details");
        }
        Ok(ast) => ast,
    };

    let mut pipeline =
        ConstantPropagation::new(&test.diagnostics).chain(Inlining::new(&test.diagnostics));
    let program = pipeline.run(program).unwrap();

    let mut expected = Program::new(ident!(root));
    expected.trace_columns.push(trace_segment!(
        0,
        "$main",
        [
            (instruction_word, 1),
            (instruction_bits, 32),
            (immediate, 1),
            (s, 1)
        ]
    ));
    expected.public_inputs.insert(
        ident!(stack_inputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(stack_inputs), 16),
    );
    expected.public_inputs.insert(
        ident!(stack_outputs),
        PublicInput::new(SourceSpan::UNKNOWN, ident!(stack_outputs), 16),
    );
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(instruction_word, Boundary::First, Type::Felt),
        int!(0)
    )));
    expected.boundary_constraints.push(enforce!(eq!(
        bounded_access!(instruction_word, Boundary::Last, Type::Felt),
        int!(2)
    )));
    for i in 0..32 {
        let access = ScalarExpr::SymbolAccess(SymbolAccess {
            span: miden_diagnostics::SourceSpan::UNKNOWN,
            name: ResolvableIdentifier::Local(ident!(instruction_bits)),
            access_type: AccessType::Index(i),
            offset: 0,
            ty: Some(Type::Felt),
        });
        expected
            .integrity_constraints
            .push(enforce!(eq!(exp!(access.clone(), int!(2)), access.clone())));
    }
    let word_sum = (1..32)
        .into_iter()
        .fold(access!("%lc0", Type::Felt), |acc, i| {
            let access = ScalarExpr::SymbolAccess(SymbolAccess {
                span: miden_diagnostics::SourceSpan::UNKNOWN,
                name: ResolvableIdentifier::Local(Identifier::new(
                    miden_diagnostics::SourceSpan::UNKNOWN,
                    crate::Symbol::intern(format!("%lc{}", i)),
                )),
                access_type: AccessType::Default,
                offset: 0,
                ty: Some(Type::Felt),
            });
            add!(acc, access)
        });
    let high_bit_sum = (33..53)
        .into_iter()
        .fold(access!("%lc32", Type::Felt), |acc, i| {
            let access = ScalarExpr::SymbolAccess(SymbolAccess {
                span: miden_diagnostics::SourceSpan::UNKNOWN,
                name: ResolvableIdentifier::Local(Identifier::new(
                    miden_diagnostics::SourceSpan::UNKNOWN,
                    crate::Symbol::intern(format!("%lc{}", i)),
                )),
                access_type: AccessType::Default,
                offset: 0,
                ty: Some(Type::Felt),
            });
            add!(acc, access)
        });
    let low_bit_sum = (54..64)
        .into_iter()
        .fold(access!("%lc53", Type::Felt), |acc, i| {
            let access = ScalarExpr::SymbolAccess(SymbolAccess {
                span: miden_diagnostics::SourceSpan::UNKNOWN,
                name: ResolvableIdentifier::Local(Identifier::new(
                    miden_diagnostics::SourceSpan::UNKNOWN,
                    crate::Symbol::intern(format!("%lc{}", i)),
                )),
                access_type: AccessType::Default,
                offset: 0,
                ty: Some(Type::Felt),
            });
            add!(acc, access)
        });
    let low_bit_sum_body = let_!(low_bit_sum = expr!(low_bit_sum) =>
        enforce!(eq!(access!(immediate, Type::Felt), add!(access!(low_bit_sum, Type::Felt), access!(high_bit_sum, Type::Felt))), when access!(s, Type::Felt)),
        enforce!(eq!(access!(instruction_bits[31], Type::Felt), int!(1)), when access!(s, Type::Felt)));
    let high_bit_sum_body = let_!(high_bit_sum = expr!(high_bit_sum)
                => let_!("%lc53" = expr!(mul!(access!(instruction_bits[20], Type::Felt), int!(1)))
                => let_!("%lc54" = expr!(mul!(access!(instruction_bits[21], Type::Felt), int!(2)))
                => let_!("%lc55" = expr!(mul!(access!(instruction_bits[22], Type::Felt), int!(4)))
                => let_!("%lc56" = expr!(mul!(access!(instruction_bits[23], Type::Felt), int!(8)))
                => let_!("%lc57" = expr!(mul!(access!(instruction_bits[24], Type::Felt), int!(16)))
                => let_!("%lc58" = expr!(mul!(access!(instruction_bits[25], Type::Felt), int!(32)))
                => let_!("%lc59" = expr!(mul!(access!(instruction_bits[26], Type::Felt), int!(64)))
                => let_!("%lc60" = expr!(mul!(access!(instruction_bits[27], Type::Felt), int!(128)))
                => let_!("%lc61" = expr!(mul!(access!(instruction_bits[28], Type::Felt), int!(256)))
                => let_!("%lc62" = expr!(mul!(access!(instruction_bits[29], Type::Felt), int!(512)))
                => let_!("%lc63" = expr!(mul!(access!(instruction_bits[30], Type::Felt), int!(1024)))
                => low_bit_sum_body))))))))))));
    let word_sum_body = let_!(word_sum = expr!(word_sum)
          => enforce!(eq!(access!(instruction_word, Type::Felt), access!(word_sum, Type::Felt))),
             let_!("%lc32" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(2048)))
             => let_!("%lc33" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(4096)))
             => let_!("%lc34" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(8192)))
             => let_!("%lc35" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(16384)))
             => let_!("%lc36" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(32768)))
             => let_!("%lc37" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(65536)))
             => let_!("%lc38" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(131072)))
             => let_!("%lc39" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(262144)))
             => let_!("%lc40" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(524288)))
             => let_!("%lc41" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(1048576)))
             => let_!("%lc42" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(2097152)))
             => let_!("%lc43" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(4194304)))
             => let_!("%lc44" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(8388608)))
             => let_!("%lc45" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(16777216)))
             => let_!("%lc46" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(33554432)))
             => let_!("%lc47" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(67108864)))
             => let_!("%lc48" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(134217728)))
             => let_!("%lc49" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(268435456)))
             => let_!("%lc50" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(536870912)))
             => let_!("%lc51" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(1073741824)))
             => let_!("%lc52" = expr!(mul!(access!(instruction_bits[31], Type::Felt), int!(2147483648)))
             => high_bit_sum_body))))))))))))))))))))),
           enforce!(eq!(access!(immediate, Type::Felt), int!(0)), when not!(access!(s, Type::Felt)))
    );

    expected.integrity_constraints.push(
        let_!("%lc0" = expr!(mul!(int!(1), access!(instruction_bits[0], Type::Felt)))
        => let_!("%lc1" = expr!(mul!(int!(2), access!(instruction_bits[1], Type::Felt)))
        => let_!("%lc2" = expr!(mul!(int!(4), access!(instruction_bits[2], Type::Felt)))
        => let_!("%lc3" = expr!(mul!(int!(8), access!(instruction_bits[3], Type::Felt)))
        => let_!("%lc4" = expr!(mul!(int!(16), access!(instruction_bits[4], Type::Felt)))
        => let_!("%lc5" = expr!(mul!(int!(32), access!(instruction_bits[5], Type::Felt)))
        => let_!("%lc6" = expr!(mul!(int!(64), access!(instruction_bits[6], Type::Felt)))
        => let_!("%lc7" = expr!(mul!(int!(128), access!(instruction_bits[7], Type::Felt)))
        => let_!("%lc8" = expr!(mul!(int!(256), access!(instruction_bits[8], Type::Felt)))
        => let_!("%lc9" = expr!(mul!(int!(512), access!(instruction_bits[9], Type::Felt)))
        => let_!("%lc10" = expr!(mul!(int!(1024), access!(instruction_bits[10], Type::Felt)))
        => let_!("%lc11" = expr!(mul!(int!(2048), access!(instruction_bits[11], Type::Felt)))
        => let_!("%lc12" = expr!(mul!(int!(4096), access!(instruction_bits[12], Type::Felt)))
        => let_!("%lc13" = expr!(mul!(int!(8192), access!(instruction_bits[13], Type::Felt)))
        => let_!("%lc14" = expr!(mul!(int!(16384), access!(instruction_bits[14], Type::Felt)))
        => let_!("%lc15" = expr!(mul!(int!(32768), access!(instruction_bits[15], Type::Felt)))
        => let_!("%lc16" = expr!(mul!(int!(65536), access!(instruction_bits[16], Type::Felt)))
        => let_!("%lc17" = expr!(mul!(int!(131072), access!(instruction_bits[17], Type::Felt)))
        => let_!("%lc18" = expr!(mul!(int!(262144), access!(instruction_bits[18], Type::Felt)))
        => let_!("%lc19" = expr!(mul!(int!(524288), access!(instruction_bits[19], Type::Felt)))
        => let_!("%lc20" = expr!(mul!(int!(1048576), access!(instruction_bits[20], Type::Felt)))
        => let_!("%lc21" = expr!(mul!(int!(2097152), access!(instruction_bits[21], Type::Felt)))
        => let_!("%lc22" = expr!(mul!(int!(4194304), access!(instruction_bits[22], Type::Felt)))
        => let_!("%lc23" = expr!(mul!(int!(8388608), access!(instruction_bits[23], Type::Felt)))
        => let_!("%lc24" = expr!(mul!(int!(16777216), access!(instruction_bits[24], Type::Felt)))
        => let_!("%lc25" = expr!(mul!(int!(33554432), access!(instruction_bits[25], Type::Felt)))
        => let_!("%lc26" = expr!(mul!(int!(67108864), access!(instruction_bits[26], Type::Felt)))
        => let_!("%lc27" = expr!(mul!(int!(134217728), access!(instruction_bits[27], Type::Felt)))
        => let_!("%lc28" = expr!(mul!(int!(268435456), access!(instruction_bits[28], Type::Felt)))
        => let_!("%lc29" = expr!(mul!(int!(536870912), access!(instruction_bits[29], Type::Felt)))
        => let_!("%lc30" = expr!(mul!(int!(1073741824), access!(instruction_bits[30], Type::Felt)))
        => let_!("%lc31" = expr!(mul!(int!(2147483648), access!(instruction_bits[31], Type::Felt)))
        => word_sum_body)))))))))))))))))))))))))))))))),
    );
    // The evaluator definition is never modified by constant propagation or inlining
    let body = vec![
        let_!(sign_bit = expr!(access!(instruction_bits[31], Type::Felt))
        => let_!(high_bit_sum = expr!(call!(sum(expr!(lc!(((i, range!(11..32))) => mul!(access!(sign_bit, Type::Felt), exp!(int!(2), access!(i, Type::Felt))))))))
        => let_!(immediate_bits = expr!(slice!(instruction_bits, 20..31, Type::Vector(11)))
        => let_!(low_bit_sum = expr!(call!(sum(expr!(lc!(((i, range!(0..11)), (immediate_bit, expr!(slice!(instruction_bits, 20..31, Type::Vector(11))))) => mul!(access!(immediate_bit, Type::Felt), exp!(int!(2), access!(i, Type::Felt))))))))
        => enforce!(eq!(access!(immediate, Type::Felt), add!(access!(low_bit_sum, Type::Felt), access!(high_bit_sum, Type::Felt)))),
           enforce!(eq!(access!(sign_bit, Type::Felt), int!(1))))))),
    ];
    expected.evaluators.insert(
        function_ident!(root, imm_reconstruction),
        EvaluatorFunction::new(
            SourceSpan::UNKNOWN,
            ident!(imm_reconstruction),
            vec![trace_segment!(
                0,
                "%2",
                [(instruction_bits, 32), (immediate, 1)]
            )],
            body,
        ),
    );

    assert_eq!(program, expected);
}
