use crate::passes::Inlining;

#[cfg(test)]
mod tests {
    use air_pass::Pass;

    use crate::ConstantValue;
    use crate::MirGraph;
    use crate::MirValue;
    use crate::Node;
    use crate::Operation;
    use crate::SpannedMirValue;

    use super::Inlining;

    #[test]
    fn test_inlining() {
        let original = MirGraph::new(vec![Node {
            op: Operation::Value(SpannedMirValue {
                span: Default::default(),
                value: MirValue::Constant(ConstantValue::Felt(1)),
            }),
        }]);
        println!("ORIGINAL: {:#?}", original);
        let mut inliner = Inlining::new();
        let result = inliner.run(original);
        println!("RESULT: {:#?}", result);
    }
}
