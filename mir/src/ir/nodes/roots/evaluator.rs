use air_types::{FunctionType, Typing};
use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{Builder, BuilderHook, Link, Node, Op, Owner, Parent, Root, Singleton};

/// A MIR Root to represent a Evaluator definition
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Root)]
pub struct Evaluator {
    // Parameters of the evaluator.
    // each parameter Identifier in the ast corresponds to a Vec<Parameter>
    pub parameters: Vec<Vec<Link<Op>>>,
    // Operations contained in the Evaluator
    pub body: Link<Vec<Link<Op>>>,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
    pub func_ty: FunctionType,
}

impl Typing for Evaluator {
    fn ty(&self) -> Option<air_types::Type> {
        self.func_ty.result()
    }
}

impl BuilderHook for Evaluator {}

impl Evaluator {
    pub fn create(
        parameters: Vec<Vec<Link<Op>>>,
        body: Vec<Link<Op>>,
        span: SourceSpan,
        func_ty: FunctionType,
    ) -> Link<Root> {
        Root::Evaluator(Self {
            parameters,
            body: Link::new(body),
            span,
            func_ty,
            ..Default::default()
        })
        .into()
    }
}

impl Parent for Evaluator {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.body.clone()
    }
}
