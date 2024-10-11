use crate::passes::Inlining;

#[cfg(test)]
mod tests {
    use air_pass::Pass;

    use crate::ConstantValue;
    use crate::MirGraph;
    use crate::MirType;
    use crate::MirValue;
    use crate::Node;
    use crate::NodeIndex;
    use crate::Operation;
    use crate::SpannedMirValue;

    use super::Inlining;

    #[test]
    fn test_inlining() {
        // double(x):
        //   y = x + x
        //   return y
        // main:
        //   x = 1
        //   return double(x)
        let original = MirGraph::new(vec![
            // stack: [1]
            Node {
                op: Operation::Value(SpannedMirValue {
                    span: Default::default(),
                    value: MirValue::Constant(ConstantValue::Felt(1)),
                }),
            },
            // body: stack[0] + stack[0]
            Node {
                op: Operation::Add(NodeIndex(0), NodeIndex(0)),
            },
            Node {
                op: Operation::Call(NodeIndex(1), vec![NodeIndex(0)]),
            },
        ]);
        println!("ORIGINAL: {:#?}", original);
        let mut inliner = Inlining::new();
        let result = inliner.run(original);
        println!("RESULT: {:#?}", result);
    }
}
