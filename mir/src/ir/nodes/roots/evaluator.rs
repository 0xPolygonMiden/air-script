use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{Builder, Link, Node, Op, Owner, OwnerId, Parent, Root, Singleton};

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
    pub owner_id: OwnerId,
    #[span]
    pub span: SourceSpan,
}

impl Evaluator {
    pub fn create(
        parameters: Vec<Vec<Link<Op>>>,
        body: Vec<Link<Op>>,
        span: SourceSpan,
    ) -> Link<Root> {
        Root::Evaluator(Self {
            parameters,
            body: Link::new(body),
            span,
            owner_id: OwnerId::next(),
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
