use ir::{TransitionConstraints, AirIR, BoundaryConstraints, Expr};
use std::collections::BTreeMap;
use crate::build_codegen_test;

use super::{super::CodeGenerator, INDENT_SPACES};

#[test]
fn boundary_constraints() {
    let air_name = "CustomAir";

    let mut first_boundary_constraints = BTreeMap::new();
    first_boundary_constraints.insert(0, Expr::Constant(0));
    let mut last_boundary_constraints = BTreeMap::new();
    last_boundary_constraints.insert(0, Expr::Constant(1));
    let main_boundary_constraints =
        BoundaryConstraints::new(first_boundary_constraints, last_boundary_constraints);
    let main_transition_constraints = TransitionConstraints::default();
    let ir = AirIR::new(
        air_name.to_string(),
        main_boundary_constraints,
        main_transition_constraints,
    );
    let scope = CodeGenerator::new(&ir);
    let source = scope.generate();

    let mut test = build_codegen_test!(&source);

    // make sure the imports are correct
    test.expect_imports();

    // --- make sure the boundary constraints are correct -----------------------------------------
    // struct definition
    
    // start struct definition
    let mut struct_def = vec!["pub struct CustomAir {".to_string()];

    // add struct fields
    let mut struct_def_fields = vec![];
    struct_def_fields.push("context: AirContext<Felt>,".to_string());

    // append struct fields with indent level 1
    indent(&mut struct_def_fields, 1);
    struct_def.extend(struct_def_fields);

    // end struct definition
    struct_def.push("}".to_string());

    // make sure the struct definition is correct
    test.expect_lines(&struct_def);

}

fn indent(lines: &mut Vec<String>, indent_level: usize) {
    let mut indent = (0..INDENT_SPACES * indent_level).map(|_| " ").collect::<String>();
    for mut line in lines {
        *line = format!("{}{}", indent, line);
    }
}

fn dedent(lines: &mut Vec<String>, indent_level: usize) {
    for mut line in lines {
        *line = line[(INDENT_SPACES * indent_level)..].to_string();
    }
}