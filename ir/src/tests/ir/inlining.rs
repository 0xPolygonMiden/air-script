use crate::passes::Inlining;

#[cfg(test)]
mod tests {
    use air_parser::ast::Identifier;
    use air_parser::ast::QualifiedIdentifier;
    use air_parser::Symbol;
    use miden_diagnostics::SourceSpan;

    use crate::graph::pretty;
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
                    Some(NodeIndex(1)),       // return
                    vec![NodeIndex(2)], // y = x + x
                ),
            },
        ]);

        println!("ORIGINAL: {}", pretty(&original, &[NodeIndex(3)]));
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
