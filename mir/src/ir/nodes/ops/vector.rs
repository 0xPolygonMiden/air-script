use air_types::*;
use miden_diagnostics::{SourceSpan, Spanned};

use crate::ir::{BackLink, Builder, BuilderHook, Child, Link, Node, Op, Owner, Parent, Singleton};

/// A MIR operation to represent a vector of MIR ops of a given size
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder, Spanned)]
#[enum_wrapper(Op)]
pub struct Vector {
    pub parents: Vec<BackLink<Owner>>,
    pub size: usize,
    pub elements: Link<Vec<Link<Op>>>,
    pub _node: Singleton<Node>,
    pub _owner: Singleton<Owner>,
    #[span]
    pub span: SourceSpan,
    pub _kind: Option<Kind>,
}

impl ScalarTypeMut for Vector {
    fn update_scalar_ty_unchecked(&mut self, new_sty: Option<ScalarType>) {
        self._kind.update_scalar_ty_unchecked(new_sty);
    }
}

impl TypeMut for Vector {
    fn update_ty_unchecked(&mut self, new_ty: Option<Type>) {
        self._kind.update_ty_unchecked(new_ty);
    }
}

impl Typing for Vector {
    fn ty(&self) -> Option<Type> {
        self._kind.ty()
    }
}

impl BuilderHook for Vector {
    fn finalize_hook(&mut self) {
        self._kind = self.elements.borrow().kind();
    }
}

impl Vector {
    pub fn create(elements: Vec<Link<Op>>, span: SourceSpan) -> Link<Op> {
        let size = elements.len();
        let mut vec = Self {
            size,
            elements: Link::new(elements),
            span,
            ..Default::default()
        };
        vec.finalize_hook();
        Op::Vector(vec).into()
    }
}

impl Parent for Vector {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.elements.clone()
    }
}

impl Child for Vector {
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
