use crate::passes::Inlining;

#[cfg(test)]
mod tests {
    use crate::graph::pretty;
    use crate::ConstantValue;
    use crate::MirGraph;
    use crate::MirType;
    use crate::MirValue;
    use crate::Node;
    use crate::NodeIndex;
    use crate::Operation;
    use crate::SpannedMirValue;

    #[test]
    fn test_inlining() {
        // double(x):
        //   y = x + x
        //   return y
        // main:
        //   x = 1
        //   return double(x)
        let original = MirGraph::new(vec![
            // double definition
            // x: Variable(Felt, 0, main)
            Node {
                // NodeIndex(0)
                op: Operation::Value(SpannedMirValue {
                    span: Default::default(),
                    value: MirValue::Variable(
                        MirType::Felt,
                        0,            // arg0
                        NodeIndex(3), // double.body
                    ),
                }),
            },
            // double.return: Variable(Felt, 1, double)
            Node {
                // NodeIndex(1)
                op: Operation::Value(SpannedMirValue {
                    span: Default::default(),
                    value: MirValue::Variable(
                        MirType::Felt,
                        1, // return because double only has one argument
                        // TODO: Define a special type for return values or add it as the last argument
                        NodeIndex(3), // double.body
                    ),
                }),
            },
            // double.body:
            Node {
                // NodeIndex(2)
                op: Operation::Add(NodeIndex(0), NodeIndex(0)),
            },
            // double.y:
            //   y = x + x
            //   return y
            Node {
                // NodeIndex(3)
                op: Operation::Definition(
                    vec![NodeIndex(0)], // x
                    Some(NodeIndex(1)), // return variable
                    vec![NodeIndex(2)], // y = x + x
                ),
            },
            // fn main() -> Felt:
            //   x = 1
            //   y = double(x)
            //   return y
            // main.return: Variable(Felt, 0, main)
            Node {
                // NodeIndex(4)
                op: Operation::Value(SpannedMirValue {
                    span: Default::default(),
                    value: MirValue::Variable(
                        MirType::Felt,
                        0,            // arg0
                        NodeIndex(0), // main.body
                    ),
                }),
            },
            // main.body:
            //   x = 1
            //   y = double(x)
            //   return y

            // x = 1
            Node {
                // NodeIndex(5)
                op: Operation::Value(SpannedMirValue {
                    span: Default::default(),
                    value: MirValue::Constant(ConstantValue::Felt(1)),
                }),
            },
            // y = double(x)
            Node {
                // NodeIndex(6)
                op: Operation::Call(NodeIndex(3), vec![NodeIndex(5)]),
            },
            //main.definition
            Node {
                // NodeIndex(7)
                op: Operation::Definition(
                    vec![],
                    Some(NodeIndex(4)), // return variable
                    vec![NodeIndex(6)], // y = double(x)
                ),
            },
        ]);

        let double = NodeIndex(3);
        let main = NodeIndex(7);
        println!("ORIGINAL: {}", pretty(&original, &[double, main]));
        // let mut inliner = Inlining::new();
        // let result = inliner.run(original);
        // println!("RESULT: {:#?}", result);
    }
}
//
// fn f0(x0: Felt) -> x1: Felt {
//   let x0 = x0 + x0;
//   return x0;
// }
//
// fn main() -> x0: Felt {
//   let x0 = 1;
//   // return f0(x0);
//   let x1 = x0 + x0;
//   return x1;
// }
