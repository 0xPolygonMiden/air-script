use air_pass::Pass;
use miden_diagnostics::{CodeMap, DiagnosticsConfig, DiagnosticsHandler, Verbosity};
use miden_diagnostics::term::termcolor::ColorChoice;

use crate::passes::ConstantPropagation;
use crate::ir::{Enf, Exp, Link, Mir, MirValue, Op, SpannedMirValue, Value, ConstantValue};

#[test]
fn constant_propagation_folds_zero_pow_zero_to_one() {
    // Build diagnostics handler
    let config = DiagnosticsConfig {
        verbosity: Verbosity::Warning,
        warnings_as_errors: true,
        no_warn: false,
        display: Default::default(),
    };
    let codemap = std::sync::Arc::new(CodeMap::new());
    let emitter = std::sync::Arc::new(miden_diagnostics::DefaultEmitter::new(ColorChoice::Auto));
    let diagnostics = DiagnosticsHandler::new(config, codemap, emitter);

    // Build MIR: enf (0^0) = ...
    let zero: Link<Op> = Value::create(SpannedMirValue {
        value: MirValue::Constant(ConstantValue::Felt(0)),
        span: Default::default(),
    });
    let pow: Link<Op> = Exp::create(zero.clone(), zero.clone(), Default::default());
    let enf: Link<Op> = Enf::create(pow, Default::default());

    let mut mir = Mir::default();
    mir.constraint_graph_mut().insert_integrity_constraints_root(enf.clone());

    // Run MIR ConstantPropagation
    let mut pass = ConstantPropagation::new(&diagnostics);
    let result = pass.run(mir).expect("MIR constant propagation should succeed");

    // Extract the transformed integrity constraint and verify 0^0 -> 1
    let roots = result
        .constraint_graph()
        .integrity_constraints_roots
        .borrow()
        .clone();
    assert_eq!(roots.len(), 1, "expected exactly one integrity constraint root");

    let enf_after = roots[0].clone();
    if let Op::Enf(e) = enf_after.borrow().as_ref() {
        let expr = e.expr.clone();
        match expr.borrow().as_ref() {
            Op::Value(v) => {
                let c = v.get_inner_const();
                assert_eq!(c, Some(1), "expected 0^0 to fold to constant 1, found {:?}", v);
            }
            other => panic!("expected Value after folding, found {:?}", other),
        }
    } else {
        panic!("expected Enf root, found {:?}", enf_after);
    }
}
