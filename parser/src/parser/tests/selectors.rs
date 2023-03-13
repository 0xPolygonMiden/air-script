use super::{build_parse_test, Identifier, IntegrityConstraint, Source, SourceSection};
use crate::ast::{Expression::*, IntegrityStmt::*, NamedTraceAccess, ConstraintType};

// SELECTORS
// ================================================================================================

#[test]
fn chained_selectors() {
    let source = "
    integrity_constraints:
        enf clk' = clk when (n1 & !n2) | !n3";
    let expected = Source(vec![SourceSection::IntegrityConstraints(vec![
        ConstraintWithSelectors(ConstraintType::IntegrityConstraint(
            IntegrityConstraint::new(
                // clk' = clk
                NamedTraceAccess(NamedTraceAccess::new(Identifier("clk".to_string()), 0, 1)),
                Elem(Identifier("clk".to_string())),
            )),
            // (n1 & !n2) | !n3
            Sub(
                Box::new(Add(
                    Box::new(Mul(
                        Box::new(Elem(Identifier("n1".to_string()))),
                        Box::new(Sub(
                            Box::new(Const(1)),
                            Box::new(Elem(Identifier("n2".to_string()))),
                        )),
                    )),
                    Box::new(Sub(
                        Box::new(Const(1)),
                        Box::new(Elem(Identifier("n3".to_string()))),
                    )),
                )),
                Box::new(Mul(
                    Box::new(Mul(
                        Box::new(Elem(Identifier("n1".to_string()))),
                        Box::new(Sub(
                            Box::new(Const(1)),
                            Box::new(Elem(Identifier("n2".to_string()))),
                        )),
                    )),
                    Box::new(Sub(
                        Box::new(Const(1)),
                        Box::new(Elem(Identifier("n3".to_string()))),
                    )),
                )),
            ),
        ),
    ])]);
    build_parse_test!(source).expect_ast(expected);
}
