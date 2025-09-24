use super::compile;

#[test]
fn test_typing() {
    let code = "
    def test

    trace_columns {
        main: [a, b],
    }

    public_inputs {
        stack_inputs: [16],
    }

    boundary_constraints {
        enf a.first = 0;
    }

    integrity_constraints {
        let b2 = as_bool(b);
        let c = select(a, b2);
        enf c = 42;
    }
    fn select(x: felt, selector: bool) -> felt {
        return x * selector;
    }
    ";
    let Ok(mir) = compile(code) else {
        panic!("Failed to compile code: {}", code);
    };
    dbg!(&mir.constraint_graph().integrity_constraints_roots);
}
