use crate::tests::compile;
#[cfg(test)]
mod tests {
    use ntest::timeout;

    use super::*;

    //use crate::graph::pretty;
    //use crate::ConstantValue;
    //use crate::ir2::Mir;
    //use crate::MirGraph;
    //use crate::MirType;
    //use crate::MirValue;
    //use crate::Node;
    //use crate::NodeIndex;
    //use crate::Operation;
    //use crate::SpannedMirValue;

    #[test]
    #[ignore]
    fn test_inlining() {
        let code = "
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
        }
        ";
        let _mir = compile(code).unwrap();
    }

    #[test]
    #[timeout(5000)]
    fn inline_deeply_nested_calls() {
        let code = "
        def DeeplyNestedInliningLoopBug

        trace_columns {
            main: [a, b, c],
        }

        public_inputs {
            x: [1],
        }

        # Utility functions (like utils module)
        fn binary_and(a: felt, b: felt) -> felt {
            return a * b;
        }

        fn binary_or(a: felt, b: felt) -> felt {
            return a + b - a * b;
        }

        fn binary_not(a: felt) -> felt {
            return 1 - a;
        }

        # Helper functions that use utilities (like flag functions)
        fn flag_current_and_next(s_next: felt) -> felt {
            return binary_not(s_next);
        }

        fn flag_last(s_next: felt) -> felt {
            return s_next;
        }

        # Function that returns array and calls multiple helper functions (like section_flags)
        fn section_flags(s_next: felt, s_start: felt, s_start_next: felt) -> felt[3] {
            let f_next_flag = flag_current_and_next(s_next);
            let f_last_flag = flag_last(s_next);

            let f_start = s_start;
            let f_next = binary_not(s_start_next);
            let f_end = binary_or(binary_and(f_next_flag, s_start_next), f_last_flag);

            return [f_start, f_next, f_end];
        }

        # Function that returns array (like block_flags)
        fn block_flags(s_block: felt) -> felt[2] {
            let f_read = binary_not(s_block);
            let f_eval = s_block;

            return [f_read, f_eval];
        }

        # Top-level evaluator that uses these functions (like section_block_flags_constraints)
        fn complex_constraint(s: felt, sstart: felt, sstart_next: felt, sblock: felt) -> felt {
            let flags = section_flags(s, sstart, sstart_next);
            let f_start = flags[0];
            let f_next = flags[1];
            let f_end = flags[2];

            let blocks = block_flags(sblock);
            let blocks_next = block_flags(sblock);  # Simulate calling with next state
            let f_read = blocks[0];
            let f_eval = blocks[1];
            let f_read_next = blocks_next[0];
            let f_eval_next = blocks_next[1];

            return f_start + f_next + f_end + f_read + f_eval + f_read_next + f_eval_next;
        }

        boundary_constraints {
            enf a.first = 0;
        }

        integrity_constraints {
            # Call complex constraint with deeply nested function calls
            enf a = complex_constraint(b, c, b, c);
        }
        ";
        let _mir = compile(code).unwrap();
    }
}
