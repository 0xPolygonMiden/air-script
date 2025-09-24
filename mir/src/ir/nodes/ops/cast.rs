use air_types::*;
use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{BackLink, Builder, BuilderHook, Child, Link, Node, Op, Owner, Parent, Singleton};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Op)]
pub struct Cast {
    pub parents: Vec<BackLink<Owner>>,
    /// The value being cast
    pub value: Link<Op>,
    ty: Option<Type>,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
}

impl BuilderHook for Cast {}

impl Cast {
    pub fn create(value: Link<Op>, ty: Option<Type>, span: SourceSpan) -> Link<Op> {
        let cast = Self { value, ty, span, ..Default::default() };
        Link::new(Op::Cast(cast))
    }
}
impl Typing for Cast {
    fn ty(&self) -> Option<Type> {
        self.ty.ty()
    }
}

impl Parent for Cast {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.value.clone()])
    }
}

impl Child for Cast {
    type Parent = Owner;
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        self.parents.clone()
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.push(parent.into());
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.retain(|p| *p != parent.clone().into());
    }
}
