use std::collections::{HashMap, HashSet};

use miden_diagnostics::SourceSpan;

use crate::{
    ir::{
        Graph, Link, MirType, Node, Op, OpInterner, OwnerId, Parameter, Parent, Vector,
        extract_boundary_roots, extract_bus_roots, extract_integrity_roots,
    },
    passes::duplicate_node_or_replace_with_interner,
    tests::compile,
};

fn count_parameters(graph: &Graph) -> usize {
    let mut seen: HashSet<usize> = HashSet::new();
    let mut count = 0usize;

    let mut roots = extract_boundary_roots(graph);
    roots.extend(extract_integrity_roots(graph));
    roots.extend(extract_bus_roots(graph));

    for root in roots {
        dfs_count_params(root, &mut seen, &mut count);
    }

    count
}

fn count_for_nodes(graph: &Graph) -> usize {
    let mut seen: HashSet<usize> = HashSet::new();
    let mut count = 0usize;

    let mut roots = extract_boundary_roots(graph);
    roots.extend(extract_integrity_roots(graph));
    roots.extend(extract_bus_roots(graph));

    for root in roots {
        dfs_count_for(root, &mut seen, &mut count);
    }

    count
}

fn dfs_count_params(node: crate::ir::Link<Node>, seen: &mut HashSet<usize>, count: &mut usize) {
    let ptr = node.get_ptr();
    if !seen.insert(ptr) {
        return;
    }

    if matches!(&*node.borrow(), Node::Parameter(_)) {
        *count += 1;
    }

    if node.as_owner().is_some() {
        for child in node.children().borrow().iter() {
            dfs_count_params(child.clone().as_node(), seen, count);
        }
    }
}

fn dfs_count_for(node: crate::ir::Link<Node>, seen: &mut HashSet<usize>, count: &mut usize) {
    let ptr = node.get_ptr();
    if !seen.insert(ptr) {
        return;
    }

    if matches!(&*node.borrow(), Node::For(_)) {
        *count += 1;
    }

    if node.as_owner().is_some() {
        for child in node.children().borrow().iter() {
            dfs_count_for(child.clone().as_node(), seen, count);
        }
    }
}

#[test]
fn unrolling_eliminates_parameters_in_nested_calls() {
    let source = "
    def test

    trace_columns {
        main: [a],
    }

    public_inputs {
        stack_inputs: [16],
    }

    boundary_constraints {
        enf a.first = 0;
    }

    integrity_constraints {
        enf a' = double_and_add_with_six(a, a);
    }

    fn double_and_add_with_six(a: felt, b: felt) -> felt {
        let c = double(a);
        let d = double(b);
        return add_six(c+d);
    }

    fn double(a: felt) -> felt {
        return 2*a;
    }

    fn add_six(a: felt) -> felt {
        let vec = [double(x) for x in 0..3];
        let vec_sum = sum(vec);
        return a + vec_sum;
    }";

    let mir = compile(source).expect("compile should succeed");
    let param_count = count_parameters(mir.constraint_graph());
    assert_eq!(
        param_count, 0,
        "expected no Parameter nodes after unrolling, found {param_count}"
    );
}

#[test]
fn unrolling_eliminates_parameters_in_nested_comprehensions_across_calls() {
    let source = "
    def test_nested_comprehensions

    trace_columns {
        main: [a],
    }

    public_inputs {
        stack_inputs: [16],
    }

    boundary_constraints {
        enf a.first = 0;
    }

    integrity_constraints {
        let v1 = helper(a);
        let v2 = helper(a);
        enf a' = v1 + v2;
    }

    fn helper(x: felt) -> felt {
        let vec = [sum([x + y for y in 0..2]) for i in 0..3];
        let vec_sum = sum(vec);
        return vec_sum;
    }";

    let mir = compile(source).expect("compile should succeed");
    let param_count = count_parameters(mir.constraint_graph());
    assert_eq!(
        param_count, 0,
        "expected no Parameter nodes after unrolling, found {param_count}"
    );
    let for_count = count_for_nodes(mir.constraint_graph());
    assert_eq!(for_count, 0, "expected no For nodes after unrolling, found {for_count}");
}

#[test]
fn duplicate_preserves_for_output_param_identity() {
    let owner_id = OwnerId::next();
    let ref_owner_id = OwnerId::next();

    let for_output = Parameter::create(0, MirType::Felt, SourceSpan::UNKNOWN);
    for_output.as_parameter_mut().unwrap().set_owner_id(owner_id);
    for_output.as_parameter_mut().unwrap().set_for_output(true);

    let regular = Parameter::create(0, MirType::Felt, SourceSpan::UNKNOWN);
    regular.as_parameter_mut().unwrap().set_owner_id(owner_id);

    let vector = Vector::create(vec![for_output, regular], SourceSpan::UNKNOWN);

    let mut replace_map: HashMap<usize, (Link<Op>, Link<Op>)> = HashMap::new();
    let mut params_for_ref_node = HashMap::new();
    let mut owner_id_map = HashMap::new();
    let mut param_cache = HashMap::new();
    let mut interner: Option<OpInterner> = None;

    duplicate_node_or_replace_with_interner(
        &mut replace_map,
        vector.clone(),
        Vec::new(),
        ref_owner_id,
        &mut params_for_ref_node,
        &mut owner_id_map,
        &mut param_cache,
        &mut interner,
    );

    let new_vector = replace_map.get(&vector.get_ptr()).unwrap().1.clone();
    let children_link = new_vector.as_vector().unwrap().children();
    let children = children_link.borrow();
    assert_eq!(children.len(), 2);

    let flags: Vec<bool> = children
        .iter()
        .map(|child| child.as_parameter().unwrap().is_for_output)
        .collect();

    assert!(
        flags.contains(&true) && flags.contains(&false),
        "expected both for_output and regular parameters to be preserved"
    );
    assert_ne!(
        children[0].get_ptr(),
        children[1].get_ptr(),
        "expected distinct parameters for for_output vs regular"
    );
}
